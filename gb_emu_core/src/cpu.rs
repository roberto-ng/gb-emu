use std::u8;

use crate::gpu::*;
use crate::cpu_registers::*;
use crate::instruction::*;

pub struct Cpu {
    registers: Registers,
    bus: MemoryBus,
    pc: u16,
    sp: u16,
    cycles: u8,
    is_halted: bool,
}

struct MemoryBus {
    memory: [u8; 0xFFFF],
    gpu: Gpu,
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; 0xFFFF],
            gpu: Gpu::new(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            VRAM_BEGIN ..= VRAM_END => {
                self.gpu.read_vram(address)
            }

            _ => {
                self.memory[address]
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            VRAM_BEGIN ..= VRAM_END => {
                self.gpu.write_vram(address, value);
            }
            
            _ => {
                self.memory[address] = value;
            }
        }
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            bus: MemoryBus::new(),
            pc: 0,
            sp: 0,
            cycles: 0,
            is_halted: false,
        }
    }

    pub fn step(&mut self) {
        let mut instruction_byte = self.bus.read_byte(self.pc);
        let prefixed = instruction_byte == 0xCB;
        if prefixed {
            // if this is a prefixed instruction, read the next byte
            instruction_byte = self.bus.read_byte(self.pc + 1);
        }

        match Instruction::from_byte(instruction_byte, prefixed) {
            Some(instruction) => {
                let (next_pc, cycles) = self.execute(&instruction);
                self.pc = next_pc;
                self.cycles += cycles;
            }
            None => panic!("Unkown instruction found: 0x{:X}", instruction_byte),
        };
    }

    fn execute(&mut self, instruction: &Instruction) -> (u16, u8) {
        if self.is_halted {
            return (self.pc, 0);
        }

        match instruction {
            Instruction::ADD(target, data) => {
                let a = self.registers.a;
                let value = self.get_arithmetic_target(target);
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

            Instruction::JP(test, data) => {
                let should_jump = self.perform_jump_test(test);
                if should_jump {
                    let low_byte = self.bus.read_byte(self.pc + 1) as u16;
                    let high_byte = self.bus.read_byte(self.pc + 2) as u16;
                    let next_pc = (high_byte << 8) | low_byte;
                    let cycles = data.action_cycles.expect("JP has no action cycles data");
                    (next_pc, cycles)
                } else {
                    let next_pc = self.pc.wrapping_add(data.bytes);
                    (next_pc, data.cycles)
                }
            }
            
            Instruction::LD(load_type, data) => {
                match load_type {
                    LoadType::Byte(target, source) => {
                        let value = self.get_load_byte_source(source);
        
                        match target {
                            LoadByteTarget::A => {
                                self.registers.a = value;
                            }
                            
                            LoadByteTarget::HLI => {
                                let address = self.registers.get_hl();
                                self.bus.write_byte(address, value);
                            }

                            _ => { panic!("TODO: implement other sources") },
                        }
        
                        let next_pc = self.pc.wrapping_add(data.bytes);
                        (next_pc, data.cycles)
                    }

                    // implement other load types
                }  
            }

            Instruction::PUSH(target, data) => {
                let value = match target {
                    StackTarget::BC => self.registers.get_bc(),
                    // TODO: support more targets
                };

                self.push(value);

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }
            
            Instruction::POP(target, data) => {
                let result = self.pop();
                match target {
                    StackTarget::BC => self.registers.set_bc(result),
                };

                let next_pc = self.pc.wrapping_add(data.bytes);
                (next_pc, data.cycles)
            }

            Instruction::CALL(test, data) => {
                // call a subroutine/function
                let should_jump = self.perform_jump_test(test);
                self.call(should_jump, data)
            }

            Instruction::RET(test, data) => {
                // return from a subroutine/function
                let should_jump = self.perform_jump_test(test);
                let next_pc = self.ret(should_jump);
                let cycles = if should_jump {
                    data.action_cycles.expect("This RET instrunction has no action cycles data")
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
        }
    }

    #[inline(always)]
    fn push(&mut self, value: u16) {
        let byte = (value & 0xFF00) >> 8;
        self.pc = self.pc.wrapping_sub(1);
        self.bus.write_byte(self.sp, byte as u8);

        let byte = value & 0xFF;
        self.pc = self.pc.wrapping_sub(1);
        self.bus.write_byte(self.sp, byte as u8)
    }

    #[inline(always)]
    fn pop(&mut self) -> u16 {
        let lsb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        let msb = self.bus.read_byte(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    #[inline(always)]
    fn call(&mut self, should_jump: bool, data: &Data) -> (u16, u8) {
        let next_pc = self.pc.wrapping_add(3);
        if should_jump {
            let cycles = data.action_cycles.expect("CALL instruction has no action cycles data");
            self.push(next_pc);
            (self.read_next_word(), cycles)
        } else {
            (next_pc, data.cycles)
        }
    }

    #[inline(always)]
    fn ret(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.pop()
        } else {
            self.pc.wrapping_add(1)
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
    const fn get_arithmetic_target(&self, target: &ArithmeticTarget) -> u8 {
        match target {
            ArithmeticTarget::A => self.registers.a,
            ArithmeticTarget::B => self.registers.b,
            ArithmeticTarget::C => self.registers.c,
            ArithmeticTarget::D => self.registers.d,
            ArithmeticTarget::E => self.registers.e,
            ArithmeticTarget::H => self.registers.h,
            ArithmeticTarget::L => self.registers.l,
        }
    }

    #[inline(always)]
    fn get_load_byte_source(&self, source: &LoadByteSource) -> u8 {
        match source {
            LoadByteSource::A => self.registers.a,
            LoadByteSource::D8 => self.read_next_byte(),
            LoadByteSource::HLI => self.bus.read_byte(self.registers.get_hl()),
            _ => { panic!("TODO: implement other sources") },
        }
    }

    #[inline(always)]
    pub fn read_next_byte(&self) -> u8 {
        self.bus.read_byte(self.pc + 1)
    }

    #[inline(always)]
    pub fn read_next_word(&self) -> u16 {
        let lsb = self.bus.read_byte(self.pc + 1) as u16;
        let msb = self.bus.read_byte(self.pc + 2) as u16;
        (msb << 8) | lsb
    }
}
