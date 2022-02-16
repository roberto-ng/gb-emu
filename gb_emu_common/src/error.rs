use crate::cartridge::cartridge_type::CartridgeType;
use std::fmt;

pub type Result<T> = std::result::Result<T, EmulationError>;

#[derive(Debug)]
pub enum EmulationError {
    InvalidMemoryRead { address: usize },
    InvalidMemoryWrite { address: usize, value: u8 },
    UnknownOpcode { opcode: u8, is_prefixed: bool },
    InvalidRomIndex { index: usize },
    InvalidRom,
    UnknownCartridgeType { code: u8 },
    UnsupportedCartridgeType { cartridge_type: CartridgeType },
    InvalidRomSizeCode { code: u8 },
    InvalidRamSizeCode { code: u8 },
    NoRom,
}

impl std::error::Error for EmulationError {}

impl fmt::Display for EmulationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidMemoryRead { address } => {
                write!(f, "Invalid read on address {address:#06X}")
            }

            Self::InvalidMemoryWrite { address, value } => {
                write!(
                    f,
                    "Invalid write read on address {address:#06X} with value {value:#06X}"
                )
            }

            Self::UnknownOpcode {
                opcode,
                is_prefixed,
            } => {
                // text that says if the opcode is prefixed or not
                let prefixed_or_not_text = if is_prefixed {
                    "prefixed"
                } else {
                    "not prefixed"
                };

                write!(f, "Unknown opcode: {opcode:#06X} {prefixed_or_not_text}")
            }

            Self::InvalidRomIndex { index } => {
                write!(f, "ROM has no index {index}")
            }

            Self::InvalidRom => {
                write!(f, "Invalid ROM")
            }

            Self::UnknownCartridgeType { code } => {
                write!(f, "Unknown cartridge type code with code {code:#04X}")
            }

            Self::InvalidRomSizeCode { code } => {
                write!(
                    f,
                    "This ROM's header informs an invalid ROM size code {code:#04X}"
                )
            }

            Self::InvalidRamSizeCode { code } => {
                write!(
                    f,
                    "This ROM's header informs an invalid RAM size code {code:#04X}"
                )
            }

            Self::UnsupportedCartridgeType { cartridge_type } => {
                write!(
                    f,
                    "The cartridge type \"{cartridge_type}\" is not supported"
                )
            }

            Self::NoRom => {
                write!(f, "No ROM loaded")
            }
        }
    }
}
