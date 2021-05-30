use crate::instruction::*;
use crate::cpu_registers::*;

struct CPU {
    registers: Registers,
}


impl CPU {
    fn execute(&mut self, instruction: &Instruction) {
        match instruction {
            Instruction::ADD {target } => {
                self.add(target, value);
            },
            _ => { panic!("Unsuported instruction.") },
        }
    }

    fn get_arithmetic_target(&self, target: ArithmeticTarget) -> u8 {
        match target {
            ArithmeticTarget::C => self.registers.c,
            _ => panic!("Unsuported target."),
        }
    }

    fn add(&mut self, target: ArithmeticTarget, value: u8) {
        let a = self.registers.a;
        let value = self.get_arithmetic_target(target);
        let (new_value, did_overflow) = a.overflowing_add(value);
        
        // set flags
        self.registers.f.zero = (new_value == 0);
        self.registers.f.subtract = false;
        self.registers.f.carry = did_overflow;
        
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.registers.f.half_carry = (self.registers.a & 0xF) + (value & 0xF) > 0xF;
    
        
        self.registers.a = new_value;
    }
}

