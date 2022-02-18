use crate::cpu_registers::*;
use crate::error::{EmulationError, Result};
use crate::instruction::*;
use crate::memory_bus::*;

pub struct Cpu {
    pub bus: MemoryBus,
    pub pc: u16,
    pub registers: Registers,
    pub sp: u16,
    is_halted: bool,
    ime: bool,     // Interrupt Master Enable Flag
    set_ime: bool, // set the IME flag only after the next instruction
    is_stopped: bool,
    //cycles: u8,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            bus: MemoryBus::new(),
            registers: Registers::new(),
            pc: 0x0100,
            sp: 0,
            is_halted: false,
            ime: false,
            set_ime: false,
            is_stopped: false,
            //cycles: 0,
        }
    }

    pub fn step(&mut self) -> Result<u32> {
        if self.is_halted {
            return Ok(1);
        }

        let mut instruction_byte = self.bus.read_byte(self.pc)?;
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            // if this is a prefixed instruction, read the next byte instead
            instruction_byte = self.bus.read_byte(self.pc.wrapping_add(1))?;
        }

        match Instruction::from_byte(instruction_byte, prefixed) {
            Some(instruction) => {
                let (next_pc, cycles) = self.execute(&instruction)?;
                self.pc = next_pc;
                Ok(cycles)
            }

            None => {
                let err = EmulationError::UnknownOpcode {
                    opcode: instruction_byte,
                    is_prefixed: prefixed,
                };
                Err(err)
            }
        }
    }

    fn execute(&mut self, instruction: &Instruction) -> Result<(u16, u32)> {
        if self.set_ime {
            self.ime = true;
            self.set_ime = false;
        }

        let result = match &instruction {
            Instruction::AdC(source, data) => {
                // Add the source value plus the carry flag to A.
                let a = self.registers.a;
                let c = if self.registers.f.carry { 1 } else { 0 };
                let value = self.get_byte_source_value(source)?;
                let (new_value, did_overflow) = a.overflowing_add(value.wrapping_add(c));

                // Set flags
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

            Instruction::Add(source, data) => {
                let a = self.registers.a;
                let value = self.get_byte_source_value(source)?;
                let (new_value, did_overflow) = a.overflowing_add(value);

                // Set flags
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

            Instruction::Add16Bits(source, target, data) => {
                let source_value = self.get_word_source_value(source)?;
                let target_value = self.get_word_target_value(target)?;
                let (new_value, did_overflow) = target_value.overflowing_add(source_value);

                // Set flags
                self.registers.f.subtract = false;
                self.registers.f.carry = did_overflow;
                self.registers.f.half_carry = (target_value & 0xF) + (source_value & 0xF) > 0xF;

                self.set_word_target_value(target, new_value)?;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::And(source, data) => {
                let a = self.registers.a;
                let value = self.get_byte_source_value(source)?;
                let new_value = a & value;
                self.registers.a = new_value;

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = false;
                self.registers.f.half_carry = true;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Bit(bit_pos, source, data) => {
                let byte = self.get_byte_source_value(source)?;
                let bit = byte & (0x01 << bit_pos);

                // Set flags
                self.registers.f.zero = bit == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = true;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::CCF(data) => {
                // Complement Carry Flag.
                self.registers.f.carry = !self.registers.f.carry;

                // Set other flags
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Cp(source, data) => {
                // Subtract the value in the byte source from A and set flags accordingly, but don't store the result.
                // This is useful for ComParing values.
                let a = self.registers.a;
                let value = self.get_byte_source_value(source)?;
                let result = a.wrapping_sub(value);

                // Set flags
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = true;
                self.registers.f.carry = value > a;
                self.registers.f.half_carry = (a & 0xF) < (value & 0xF);

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Cpl(data) => {
                // ComPLement accumulator (A = ~A).
                self.registers.a = !self.registers.a;

                // Set flags
                self.registers.f.subtract = true;
                self.registers.f.half_carry = true;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::DAA(data) => {
                // Decimal Adjust Accumulator to get a correct BCD representation after an arithmetic instruction.
                let a = self.registers.a;
                let mut new_value = a;
                let mut carry = false;
                if !self.registers.f.subtract {
                    if self.registers.f.half_carry || (a & 0x0F) > 9 {
                        new_value = new_value.wrapping_add(0x06);
                    }
                    if self.registers.f.carry || a > 0x9F {
                        new_value = new_value.wrapping_add(0x60);
                        carry = true;
                    }
                } else {
                    if self.registers.f.half_carry {
                        new_value = (a.wrapping_sub(0x06)) & 0xFF;
                    } else {
                        new_value = new_value.wrapping_sub(0x60);
                    }
                }
                new_value &= 0xFF;
                self.registers.a = new_value;

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.carry = carry;
                self.registers.f.half_carry = true;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Dec(target, data) => {
                let value = self.get_byte_target_value(target)?;
                let new_value = value.wrapping_sub(1);

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = (value & 0xF) + (1 & 0xF) > 0xF;

                self.set_byte_target_value(target, new_value)?;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Dec16Bits(target, data) => {
                let value = self.get_word_target_value(target)?;
                let new_value = value.wrapping_sub(1);
                self.set_word_target_value(target, new_value)?;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::DI(data) => {
                // Disable Interrupts by clearing the IME flag.
                self.ime = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::EI(data) => {
                // Enable Interrupts by setting the IME flag. The flag is only set after the instruction following EI.
                self.set_ime = true;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Inc(target, data) => {
                let value = self.get_byte_target_value(target)?;
                let new_value = value.wrapping_add(1);

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.half_carry = (value & 0xF) + (1 & 0xF) > 0xF;

                self.set_byte_target_value(target, new_value)?;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Inc16Bits(target, data) => {
                let value = self.get_word_target_value(target)?;
                let new_value = value.wrapping_add(1);
                self.set_word_target_value(target, new_value)?;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Or(source, data) => {
                // Bitwise OR
                let a = self.registers.a;
                let value = self.get_byte_source_value(source)?;
                let new_value = a | value;
                self.registers.a = new_value;

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = false;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Res(bit_pos, target, data) => {
                // Set bit u3 in target to 0. Bit 0 is the rightmost one, bit 7 the leftmost one.
                let byte = self.get_byte_target_value(target)?;
                let result = byte & !(1 << bit_pos);
                self.set_byte_target_value(target, result)?;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::RL(target, data) => {
                // Rotate bits in target left through carry.
                let c = if self.registers.f.carry { 1 } else { 0 };
                let value = self.get_byte_target_value(target)?;
                let new_value = value.wrapping_shl(1) | c;
                self.set_byte_target_value(target, new_value)?;

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = (value & 0x80) == 0x80;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::RLC(target, data) => {
                // Rotate target left.
                let value = self.get_byte_target_value(target)?;
                let new_c = (value & 0x80) == 0x80;
                let new_value = value.wrapping_shl(1) | if new_c { 1 } else { 0 };
                self.set_byte_target_value(target, new_value)?;

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = new_c;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::RR(target, data) => {
                // Rotate bits in target right through carry.
                let c = self.registers.f.carry;
                let value = self.get_byte_target_value(target)?;
                let new_value = value.wrapping_shr(1) | if c { 0x80 } else { 0x00 };
                self.set_byte_target_value(target, new_value)?;

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = (value & 0x01) == 0x01;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::RRC(target, data) => {
                // Rotate target right.
                let value = self.get_byte_target_value(target)?;
                let new_c = (value & 0x01) == 0x01;
                let new_value = value.wrapping_shr(1) | if new_c { 0x80 } else { 0x00 };
                self.set_byte_target_value(target, new_value)?;

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = new_c;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::RST(vec, data) => {
                // Call address vec. This is a shorter and faster equivalent to CALL
                // for suitable values of vec (0x00, 0x08, 0x10, 0x18, 0x20, 0x28, 0x30, and 0x38).
                let next_instruction = self.pc.wrapping_add(data.bytes);
                self.push(next_instruction)?;

                let next_pc = *vec as u16;
                (next_pc, data.cycles)
            }

            Instruction::SbC(source, data) => {
                // Subtract the value in source and the carry flag from A.
                let a = self.registers.a;
                let carry = if self.registers.f.carry { 1 } else { 0 };
                let value = self.get_byte_source_value(source)?;
                let (result, did_overflow) = a.overflowing_sub(value.wrapping_add(carry));
                self.registers.a = result;

                // Set flags
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = true;
                self.registers.f.carry = did_overflow;
                self.registers.f.half_carry = ((result ^ value ^ a) & 0x10) == 0x10;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::SCF(data) => {
                // Set Carry Flag.
                self.registers.f.carry = true;

                // Set other flags
                self.registers.f.subtract = false;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Set(bit_pos, target, data) => {
                //  Set bit u3 in target to 1. Bit 0 is the rightmost one, bit 7 the leftmost one.
                let byte = self.get_byte_target_value(target)?;
                let result = byte | (0b00000001 << bit_pos);
                self.set_byte_target_value(target, result)?;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::SLA(target, data) => {
                // Shift Left Arithmetically
                let value = self.get_byte_target_value(target)?;
                let new_value = value.wrapping_shl(1);
                self.set_byte_target_value(target, new_value)?;

                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = (value & 0x80) == 0x80;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::SRA(target, data) => {
                // Shift Left Arithmetically
                let value = self.get_byte_target_value(target)?;
                let new_value = value.wrapping_shr(1) | (value & 0x80);
                self.set_byte_target_value(target, new_value)?;

                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = (value & 0x01) == 0x01;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::SRL(target, data) => {
                // Shift Left Arithmetically
                let value = self.get_byte_target_value(target)?;
                let new_value = value.wrapping_shr(1);
                self.set_byte_target_value(target, new_value)?;

                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = (value & 0x01) == 0x01;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Stop(data) => {
                self.is_stopped = true;

                // The DIV - Divider Register is reset when executing the stop instruction,
                // and only begins ticking again once stop mode ends
                self.bus.reset_divider_register();

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Sub(source, data) => {
                let a = self.registers.a;
                let value = self.get_byte_source_value(source)?;
                let result = a.wrapping_sub(value);
                self.registers.a = result;

                // Set flags
                self.registers.f.zero = result == 0;
                self.registers.f.subtract = true;
                self.registers.f.half_carry = (a & 0xF) < (value & 0xF);
                self.registers.f.carry = value > a;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Swap(target, data) => {
                //  Swap the upper 4 bits in the target and the lower 4 ones.
                let value = self.get_byte_target_value(target)?;
                let upper = (0xF0 & value) >> 4;
                let lower = (0x0F & value) << 4;
                let new_value = upper ^ lower;

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = false;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::XOr(source, data) => {
                // Bitwise XOR
                let a = self.registers.a;
                let value = self.get_byte_source_value(source)?;
                let new_value = a ^ value;
                self.registers.a = new_value;

                // Set flags
                self.registers.f.zero = new_value == 0;
                self.registers.f.subtract = false;
                self.registers.f.carry = false;
                self.registers.f.half_carry = false;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Jp(test, source, data) => {
                let should_jump = self.perform_jump_test(test);
                if should_jump {
                    let next_pc = self.get_word_source_value(source)?;
                    let cycles = data.get_action_cycles();
                    (next_pc, cycles)
                } else {
                    let next_pc = self.pc.wrapping_add(data.bytes);
                    (next_pc, data.cycles)
                }
            }

            Instruction::JR(test, data) => {
                // Relative Jump by adding e8 to the address of the instruction following the JR
                let should_jump = self.perform_jump_test(test);
                let next_instruction = self.pc.wrapping_add(data.bytes);
                let byte = self.read_next_byte()? as i8;

                if should_jump {
                    let next_pc = next_instruction.wrapping_add(byte as u16);
                    (next_pc, data.get_action_cycles())
                } else {
                    (next_instruction, data.cycles)
                }
            }

            Instruction::Ld(load_type, data) => {
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

            Instruction::Push(target, data) => {
                let value = self.get_rr_value(target);
                self.push(value)?;

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Pop(target, data) => {
                let result = self.pop()?;
                self.set_rr_value(target, result);

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::Call(test, data) => {
                // call a subroutine/function
                let should_jump = self.perform_jump_test(test);
                let next_pc = self.pc.wrapping_add(3);
        
                if should_jump {
                    self.push(next_pc)?;

                    let cycles = data.get_action_cycles();
                    let next_pc = self.read_next_word()?;
                    (next_pc, cycles)
                } else {
                    (next_pc, data.cycles)
                }
            }

            Instruction::Ret(test, data) => {
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

            Instruction::RetI(data) => {
                // Return from subroutine and enable interrupts. This is basically equivalent
                // to executing EI then RET, meaning that IME is set right after this instruction.
                let next_pc = self.ret(true)?;
                self.ime = true;
                (next_pc, data.cycles)
            }

            Instruction::NoOp(data) => {
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
        let msb = (value & 0xFF00) >> 8;
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, msb as u8)?;

        let lsb = value & 0xFF;
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, lsb as u8)?;

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
            ByteSource::Register(r) => self.get_r_value(r),

            ByteSource::Registers(rr) => {
                let word = self.get_rr_value(rr);
                self.bus.read_byte(word)?
            }

            ByteSource::Immediate8 => self.read_next_byte()?,

            ByteSource::Direct => {
                let address = self.read_next_word()?;
                self.bus.read_byte(address)?
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

            ByteSource::FF00PlusU8 => {
                let byte = self.read_next_byte()? as u16;
                let address = 0xFF00 + byte;
                self.bus.read_byte(address)?
            }
        };

        Ok(byte)
    }

    #[inline(always)]
    fn get_byte_target_value(&mut self, target: &ByteTarget) -> Result<u8> {
        match &target {
            ByteTarget::Register(r) => Ok(self.get_r_value(r)),

            ByteTarget::Registers(rr) => {
                let word = self.get_rr_value(rr);
                self.bus.read_byte(word)
            }

            ByteTarget::Immediate8 => {
                let word = self.read_next_word()?;
                self.bus.read_byte(word)
            }

            ByteTarget::Direct => {
                let address = self.read_next_word()?;
                self.bus.read_byte(address)
            }

            ByteTarget::HL | ByteTarget::HLI | ByteTarget::HLD => {
                let hl = self.registers.get_hl();
                self.bus.read_byte(hl)
            }

            ByteTarget::FF00PlusC => {
                let address = 0xFF00 + (self.registers.c as u16);
                self.bus.read_byte(address)
            }

            ByteTarget::FF00PlusU8 => {
                let byte = self.read_next_byte()? as u16;
                let address = 0xFF00 + byte;
                self.bus.read_byte(address)
            }
        }
    }

    #[inline(always)]
    fn set_byte_target_value(&mut self, target: &ByteTarget, value: u8) -> Result<()> {
        match &target {
            ByteTarget::Register(r) => {
                self.set_r_value(r, value);
            }

            ByteTarget::Registers(rr) => {
                let word = self.get_rr_value(rr);
                self.bus.write_byte(word, value)?;
            }

            ByteTarget::Immediate8 => {
                let word = self.read_next_word()?;
                self.bus.write_byte(word, value)?;
            }

            ByteTarget::Direct => {
                let address = self.read_next_word()?;
                self.bus.write_byte(address, value)?;
            }

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

            ByteTarget::FF00PlusU8 => {
                let byte = self.read_next_byte()? as u16;
                let address = 0xFF00 + byte;
                self.bus.write_byte(address, value)?;
            }
        }

        Ok(())
    }

    #[inline(always)]
    pub fn get_word_source_value(&self, source: &WordSource) -> Result<u16> {
        match &source {
            WordSource::Registers(rr) => Ok(self.get_rr_value(rr)),

            WordSource::Immediate16 => self.read_next_word(),

            WordSource::SP => Ok(self.sp),

            WordSource::SpPlusI8 => Ok(self.sp.wrapping_add(8)),
        }
    }

    #[inline(always)]
    pub fn get_word_target_value(&mut self, target: &WordTarget) -> Result<u16> {
        match &target {
            WordTarget::Registers(rr) => Ok(self.get_rr_value(rr)),

            WordTarget::Direct => {
                let address = self.read_next_word()?;
                let lsb = self.bus.read_byte(address)? as u16;
                let msb = self.bus.read_byte(address)? as u16;
                let value = (msb << 8) | lsb;
                Ok(value)
            }

            WordTarget::SP => Ok(self.sp),
        }
    }

    #[inline(always)]
    pub fn set_word_target_value(&mut self, target: &WordTarget, value: u16) -> Result<()> {
        match &target {
            WordTarget::Registers(rr) => {
                self.set_rr_value(rr, value);
            }

            WordTarget::Direct => {
                let address = self.read_next_word()?;
                let lsb = (0x00FF & value) as u8;
                let msb = ((0xFF00 & value) >> 8) as u8;

                self.bus.write_byte(address, lsb)?;
                self.bus.write_byte(address.wrapping_add(1), msb)?;
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
        self.bus.read_byte(self.pc.wrapping_add(1))
    }

    #[inline(always)]
    pub fn read_next_word(&self) -> Result<u16> {
        let lsb = self.bus.read_byte(self.pc.wrapping_add(1))? as u16;
        let msb = self.bus.read_byte(self.pc.wrapping_add(2))? as u16;
        let next_word = (msb << 8) | lsb;

        Ok(next_word)
    }
}
