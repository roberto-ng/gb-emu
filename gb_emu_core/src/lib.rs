pub mod memory_bus;
pub mod cpu;
pub mod cpu_registers;
pub mod instruction;
pub mod gpu;
pub mod cartridge;

use std::fmt;

pub type Result<T> = std::result::Result<T, EmulationError>;

pub enum EmulationError {
    InvalidMemoryRead { address: usize },
    InvalidMemoryWrite { address: usize, value: u8 },
    UnknownOpcode { opcode: u8, is_prefixed: bool },
}

impl fmt::Display for EmulationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &EmulationError::InvalidMemoryRead { address } => {
                write!(f, "Invalid cartridge read on address {:#06X}", address)
            }

            &EmulationError::InvalidMemoryWrite { address, value } => {
                write!(f, "Invalid write read on address {:#06X} with value {:#06X}", address, value)
            }

            &EmulationError::UnknownOpcode { opcode, is_prefixed } => {
                // text that says if the opcode is prefixed or not
                let prefixed_or_not_text = if is_prefixed {
                    "prefixed"
                } else {
                    "not prefixed"
                };

                write!(f, "Unknown opcode: {:#06X} {}", opcode, prefixed_or_not_text)
            }
        }
    }
} 

#[cfg(test)]
mod tests {
    use crate::cpu::Cpu;

    #[test]
    fn it_works() {
        let _cpu = Cpu::new();
        assert_eq!(2 + 2, 4);
    }
}
