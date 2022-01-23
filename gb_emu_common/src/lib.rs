pub mod memory_bus;
pub mod cpu;
pub mod cpu_registers;
pub mod instruction;
pub mod gpu;
pub mod cartridge;

use std::fmt;

use cartridge::cartridge_type::CartridgeType;

pub type Result<T> = std::result::Result<T, EmulationError>;

#[derive(Debug)]
pub enum EmulationError {
    InvalidMemoryRead { address: usize },
    InvalidMemoryWrite { address: usize, value: u8 },
    UnknownOpcode { opcode: u8, is_prefixed: bool },
    InvalidRomIndex { index: usize, },
    InvalidRom,
    UnknownCartridgeType { code: u8, },
    UnsupportedCartridgeType { cartridge_type: CartridgeType },
    InvalidRomSizeCode { code: u8, },
    InvalidRamSizeCode { code: u8, },
}

impl fmt::Display for EmulationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Self::InvalidMemoryRead { address } => {
                write!(f, "Invalid cartridge read on address {:#06X}", address)
            }

            &Self::InvalidMemoryWrite { address, value } => {
                write!(f, "Invalid write read on address {:#06X} with value {:#06X}", address, value)
            }

            &Self::UnknownOpcode { opcode, is_prefixed } => {
                // text that says if the opcode is prefixed or not
                let prefixed_or_not_text = if is_prefixed {
                    "prefixed"
                } else {
                    "not prefixed"
                };

                write!(f, "Unknown opcode: {:#06X} {}", opcode, prefixed_or_not_text)
            }

            &Self::InvalidRomIndex { index } => {
                write!(f, "ROM has no index {}", index)
            }

            &Self::InvalidRom => {
                write!(f, "Invalid ROM")
            }

            &Self::UnknownCartridgeType { code } => {
                write!(f, "Unknown cartridge type code with code {:#04X}", code)
            }

            &Self::InvalidRomSizeCode { code } => {
                write!(f, "This ROM's header informs an invalid ROM size code {:#04X}", code)
            }

            &Self::InvalidRamSizeCode { code } => {
                write!(f, "This ROM's header informs an invalid RAM size code {:#04X}", code)
            }

            &Self::UnsupportedCartridgeType { cartridge_type } => {
                write!(f, "The cartridge type \"{cartridge_type}\" is not supported")
            }
        }
    }
} 

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
