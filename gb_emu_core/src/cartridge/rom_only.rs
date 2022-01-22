use crate::cartridge::*;
use crate::{Result, EmulationError};

pub struct RomOnlyCartridge {
    rom_bank_0: [u8; ROM_BANK_0_SIZE],
    rom_bank_n: [u8; ROM_BANK_N_SIZE],
}

impl RomOnlyCartridge {
    pub fn new() -> RomOnlyCartridge {
        RomOnlyCartridge {
            rom_bank_0: [0; ROM_BANK_0_SIZE],
            rom_bank_n: [0; ROM_BANK_N_SIZE],
        }
    }
}

impl Cartridge for RomOnlyCartridge {
    fn read_byte_rom(&self, address: usize) -> Result<u8> {
        match address {
            ROM_BANK_0_START ..= ROM_BANK_0_END => {
                Ok(self.rom_bank_0[address])
            }

            ROM_BANK_N_START ..= ROM_BANK_N_END => {
                Ok(self.rom_bank_n[address])
            }

            _ => {
                Err(EmulationError::InvalidMemoryRead { address })
            }
        }
    }

    fn write_byte_rom(&mut self, address: usize, value: u8) -> Result<()> {
        Err(EmulationError::InvalidMemoryWrite { address, value })
    }

    fn read_byte_external_ram(&self, address: usize) -> Result<u8> {
        Err(EmulationError::InvalidMemoryRead { address })
    }

    fn write_byte_external_ram(&mut self, address: usize, value: u8) -> Result<()> {
        Err(EmulationError::InvalidMemoryWrite { address, value })
    }    
}