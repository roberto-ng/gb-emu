use crate::instruction::Instruction;
use crate::cpu_registers::*;

struct CPU {
    registers: Registers,
}


impl CPU {
    fn execute(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::ADD(target) => {
                match target {
                    ArithmeticTarget::C => {
                        let value = self.registers.c;
                        let new_value = self.add(value);
                        self.registers.a = new_value;
                    }            
                    _ => {} // TODO
                }
            }
            _ => { /* TODO: support more instructions */ }
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let a = self.registers.a;
        let (new_value, did_overflow) = a.overflowing_add(value);
        
        // set flags
        self.registers.f.zero = (new_value == 0);
        self.registers.subtract = false;
        self.registers.carry = did_overflow;
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;

        
        new_value
    }
}