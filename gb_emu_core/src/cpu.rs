use std::u8;

use crate::cpu_registers::*;
use crate::instruction::{
    ArithmeticTarget, 
    Info as InstructionInfo, 
    Instruction, 
    JumpTest
};

pub struct Cpu {
    registers: Registers,
    bus: MemoryBus,
    pc: u16,
}

struct MemoryBus {
    memory: [u8; 0xFFFF],
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; 0xFFFF],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            bus: MemoryBus::new(),
            pc: 0,
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
                let next_pc = self.execute(&instruction);
                self.pc = next_pc;
            }
            None => panic!("Unkown instruction found: 0x{:X}", instruction_byte),
        };
    }

    fn execute(&mut self, instruction: &Instruction) -> u16 {
        match instruction {
            Instruction::ADD(target, info) => self.add(target, info),
            Instruction::JP(test, info) => self.jp(test, info),
            //_ => self.pc,
        }
    }

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

    fn add(&mut self, target: &ArithmeticTarget, info: &InstructionInfo) -> u16 {
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
        self.pc.wrapping_add(info.bytes as u16)
    }

    fn jp(&mut self, test: &JumpTest, info: &InstructionInfo) -> u16 {
        let should_jump = match test {
            JumpTest::NotZero => !self.registers.f.zero,
            JumpTest::NotCarry => !self.registers.f.carry,
            JumpTest::Zero => self.registers.f.zero,
            JumpTest::Carry => self.registers.f.carry,
            JumpTest::Always => true,
        };

        if should_jump {
            let low_byte = self.bus.read_byte(self.pc + 1) as u16;
            let high_byte = self.bus.read_byte(self.pc + 2) as u16;
            (high_byte << 8) | low_byte
        } else {
            self.pc.wrapping_add(info.bytes as u16)
        }
    }
}
