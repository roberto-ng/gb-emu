use crate::EmulationError;
use crate::cpu_registers::*;
use crate::instruction::*;
use crate::memory_bus::*;
use crate::Result;

pub struct Cpu {
    registers: Registers,
    bus: MemoryBus,
    pc: u16,
    sp: u16,
    cycles: u8,
    is_halted: bool,
}

impl Cpu {
    pub fn new(rom: Vec<u8>) -> Result<Cpu> {
        Ok(
            Cpu {
                registers: Registers::new(),
                bus: MemoryBus::new(rom)?,
                pc: 0,
                sp: 0,
                cycles: 0,
                is_halted: false,
            }
        )
    }

    pub fn step(&mut self) -> Result<()> {
        let mut instruction_byte = self.bus.read_byte(self.pc)?;
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            // if this is a prefixed instruction, read the next byte instead
            instruction_byte = self.bus.read_byte(self.pc + 1)?;
        }

        match Instruction::from_byte(instruction_byte, prefixed) {
            Some(instruction) => {
                let (next_pc, cycles) = self.execute(&instruction)?;
                self.pc = next_pc;
                self.cycles += cycles;
                Ok(())
            }
            None => {
                let err = EmulationError::UnknownOpcode {
                    opcode: instruction_byte,
                    is_prefixed: prefixed,
                };
                Err(err)
            },
        }
    }

    fn execute(&mut self, instruction: &Instruction) -> Result<(u16, u8)> {
        if self.is_halted {
            return Ok((self.pc, 0));
        }

        let result = match instruction {
            Instruction::ADD(target, data) => {
                let a = self.registers.a;
                let value = self.get_r_value(target); // read register
                let (new_value, did_overflow) = a.overflowing_add(value);

                // set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = did_overflow;

                // Half Carry is set if adding the lower nibbles of the value and register A
                // together result in a value bigger than 0xF. If the result is larger than 0xF
                // than the addition caused a carry from the lower nibble to the upper nibble.
                self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;

                self.registers.a = new_value;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::JP(test, _source, data) => {
                let should_jump = self.perform_jump_test(test);
                if should_jump {
                    let low_byte = self.bus.read_byte(self.pc + 1)? as u16;
                    let high_byte = self.bus.read_byte(self.pc + 2)? as u16;
                    let next_pc = (high_byte << 8) | low_byte;
                    let cycles = data.get_action_cycles();
                    (next_pc, cycles)
                } else {
                    let next_pc = self.pc.wrapping_add(data.bytes);
                    (next_pc, data.cycles)
                }
            }
            
            Instruction::LD(load_type, data) => {
                match &load_type {
                    LoadType::Byte(target, source) => {
                        let value = self.get_byte_source_value(source)?;
                        self.set_byte_target_value(target, value)?;        
                    }

                    LoadType::Word(target, source) => {
                        let value = self.get_word_source_value(source)?;
                        self.set_word_target_value(target, value)?;

                    }
                }  
                
                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::PUSH(target, data) => {
                let value =  self.get_rr_value(target);
                self.push(value)?;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }
            
            Instruction::POP(target, data) => {
                let result = self.pop()?;
                self.set_rr_value(target, result);

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::CALL(test, data) => {
                // call a subroutine/function
                let should_jump = self.perform_jump_test(test);
                self.call(should_jump, data)?
            }

            Instruction::RET(test, data) => {
                // return from a subroutine/function
                let should_jump = self.perform_jump_test(test);
                let next_pc = self.ret(should_jump)?;
                let cycles = if should_jump {
                    data.get_action_cycles()
                } else {
                    data.cycles
                };

                (next_pc, cycles)
            }

            Instruction::NOP(data) => {
                // do nothing ¯\_(ツ)_/¯
                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Halt(data) => {
                self.is_halted = true;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }
        };

        Ok(result)
    }

    #[inline(always)]
    fn push(&mut self, value: u16) -> Result<()> {
        let byte = (value & 0xFF00) >> 8;
        self.pc = self.pc.wrapping_sub(1);
        self.bus.write_byte(self.sp, byte as u8)?;

        let byte = value & 0xFF;
        self.pc = self.pc.wrapping_sub(1);
        self.bus.write_byte(self.sp, byte as u8)?;

        Ok(())
    }

    #[inline(always)]
    fn pop(&mut self) -> Result<u16> {
        let lsb = self.bus.read_byte(self.sp)? as u16;
        self.sp = self.sp.wrapping_add(1);

        let msb = self.bus.read_byte(self.sp)? as u16;
        self.sp = self.sp.wrapping_add(1);

        let result = (msb << 8) | lsb;
        Ok(result)
    }

    #[inline(always)]
    fn call(&mut self, should_jump: bool, data: &Data) -> Result<(u16, u8)> {
        let next_pc = self.pc.wrapping_add(3);
        let result = if should_jump {
            self.push(next_pc)?;

            let cycles = data.get_action_cycles();
            (self.read_next_word()?, cycles)
        } else {
            (next_pc, data.cycles)
        };

        Ok(result)
    }

    #[inline(always)]
    fn ret(&mut self, should_jump: bool) -> Result<u16> {
        if should_jump {
            self.pop()
        } else {
            Ok(self.pc.wrapping_add(1))
        }
    }

    #[inline(always)]
    const fn perform_jump_test(&self, test: &JumpTest) -> bool {
        match test {
            JumpTest::NotZero => !self.registers.f.zero,
            JumpTest::NotCarry => !self.registers.f.carry,
            JumpTest::Zero => self.registers.f.zero,
            JumpTest::Carry => self.registers.f.carry,
            JumpTest::Always => true,
        }
    }

    #[inline(always)]
    fn get_byte_source_value(&mut self, source: &ByteSource) -> Result<u8> {
        let byte = match &source {
            ByteSource::Register(r) => {
                self.get_r_value(r)
            }
            
            ByteSource::Immediate8 => {
                self.read_next_byte()?
            }
            
            ByteSource::HL => {
                let hl = self.registers.get_hl();
                self.bus.read_byte(hl)?
            }
            
            ByteSource::HLI => {
                let hl = self.registers.get_hl();
                self.registers.set_hl(hl.wrapping_add(1)); // increment HL
                self.bus.read_byte(hl)?
            }
            
            ByteSource::HLD => {
                let hl = self.registers.get_hl();
                self.registers.set_hl(hl.wrapping_sub(1)); // decrement HL
                self.bus.read_byte(hl)?
            }
            
            ByteSource::FF00PlusC => {
                let address = 0xFF00 + (self.registers.c as u16);
                self.bus.read_byte(address)?
            }
        };

        Ok(byte)
    }

    #[inline(always)]
    fn set_byte_target_value(&mut self, target: &ByteTarget, value: u8) -> Result<()> {
        match target {
            ByteTarget::Register(r) => {
                self.set_r_value(r, value);
            }

            ByteTarget::Immediate8 => {
                let word = self.read_next_word()?;
                self.bus.write_byte(word, value)?;
            },

            ByteTarget::HL => {
                let hl = self.registers.get_hl();
                self.bus.write_byte(hl, value)?;
            }
            
            ByteTarget::HLI => {
                let hl = self.registers.get_hl();
                self.registers.set_hl(hl.wrapping_add(1)); // increment HL
                self.bus.write_byte(hl, value)?;
            }

            ByteTarget::HLD => {
                let hl = self.registers.get_hl();
                self.registers.set_hl(hl.wrapping_sub(1)); // decrement HL
                self.bus.write_byte(hl, value)?;
            }

            ByteTarget::FF00PlusC => {
                let address = 0xFF00 + (self.registers.c as u16);
                self.bus.write_byte(address, value)?;
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub fn get_word_source_value(&self, source: &WordSource) -> Result<u16> {
        match &source {
            WordSource::HL => {
                Ok(self.registers.get_hl())
            }

            WordSource::Immediate16 => {
                self.read_next_word()
            }

            WordSource::SP => {
                Ok(self.sp)
            }

            WordSource::SpPlusI8 => {
                Ok(self.sp + 8)
            }
        }
    }

    #[inline(always)]
    pub fn set_word_target_value(&mut self, target: &WordTarget, value: u16) -> Result<()> {
        match &target {
            WordTarget::Direct => {
                let address = self.read_next_word()?;
                let lsb = (0x00FF & value) as u8;
                let msb = ((0xFF00 & value) >> 8) as u8;
                
                self.bus.write_byte(address, lsb)?;
                self.bus.write_byte(address + 1, msb)?;
            }

            WordTarget::HL => {
                self.registers.set_hl(value);
            }

            WordTarget::SP => {
                self.sp = value;
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub fn get_r_value(&self, r: &R) -> u8 {
        match r {
            R::A => self.registers.a,
            R::B => self.registers.b,
            R::C => self.registers.c,
            R::D => self.registers.d,
            R::E => self.registers.e,
            R::F => self.registers.f.into(),
            R::H => self.registers.h,
            R::L => self.registers.l,
        }
    }

    #[inline(always)]
    pub fn set_r_value(&mut self, r: &R, value: u8) {
        match r {
            R::A => {
                self.registers.a = value;
            }

            R::B => {
                self.registers.b = value;
            }

            R::C => {
                self.registers.c = value;
            }

            R::D => {
                self.registers.d = value;
            }

            R::E => {
                self.registers.e = value;
            }

            R::F => {
                self.registers.f = value.into();
            }

            R::H => {
                self.registers.h = value;
            }

            R::L => {
                self.registers.l = value;
            }
        }
    }

    #[inline(always)]
    pub fn get_rr_value(&self, rr: &RR) -> u16 {
        match &rr {
            RR::AF => self.registers.get_af(),
            RR::BC => self.registers.get_bc(),
            RR::DE => self.registers.get_de(),
            RR::HL => self.registers.get_hl(),
        }
    }

    #[inline(always)]
    pub fn set_rr_value(&mut self, rr: &RR, value: u16) {
        match &rr {
            RR::AF => self.registers.set_af(value),
            RR::BC => self.registers.set_bc(value),
            RR::DE => self.registers.set_de(value),
            RR::HL => self.registers.set_hl(value),
        };
    }

    #[inline(always)]
    pub fn read_next_byte(&self) -> Result<u8> {
        self.bus.read_byte(self.pc + 1)
    }

    #[inline(always)]
    pub fn read_next_word(&self) -> Result<u16> {
        let lsb = self.bus.read_byte(self.pc + 1)? as u16;
        let msb = self.bus.read_byte(self.pc + 2)? as u16;
        let next_word = (msb << 8) | lsb;

        Ok(next_word)
    }
}
