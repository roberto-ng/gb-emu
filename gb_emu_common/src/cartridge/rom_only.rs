use crate::cartridge::*;
use self::header::*;
use self::cartridge_type::*;
use crate::{Result, EmulationError};

pub struct RomOnlyCartridge {
    rom: Vec<u8>,
    ram: Option<RamBank>,
    header: Header,
}

impl RomOnlyCartridge {
    pub fn new(rom: Vec<u8>, header: Header) -> Result<RomOnlyCartridge> {
        let rom_size = ROM_BANK_SIZE * 2;
        if rom.len() != rom_size || header.rom_bank_amount != 2 {
            // this file has an invalid size
            return Err(EmulationError::InvalidRom)
        }
    
        let has_ram = header.ram_bank_amount > 0;
        let ram = if has_ram {
            Some([0; RAM_BANK_SIZE])
        } else {
            None
        };

        Ok(
            RomOnlyCartridge {
                rom,
                ram,
                header,
            }
        )
    }
}

impl Cartridge for RomOnlyCartridge {
    fn read_byte_rom(&self, address: usize) -> Result<u8> {
        match address {
            ROM_BANK_0_START ..= ROM_BANK_N_END => {
                Ok(self.rom[address])
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
        let pos = address - EXTERNAL_RAM_START;
        match &self.ram {
            Some(ram) => Ok(ram[pos]),
            None => Err(EmulationError::InvalidMemoryRead { address }),
        }
    }

    fn write_byte_external_ram(&mut self, address: usize, value: u8) -> Result<()> {
        let pos = address - EXTERNAL_RAM_START;
        match &mut self.ram {
            Some(ram) => {
                ram[pos] = value;
                Ok(())
            },
            None => Err(EmulationError::InvalidMemoryWrite { address, value })
        }
    }

    fn get_header(&self) -> Header {
        self.header.clone()
    }

    fn get_ram_banks(&self) -> Vec<RamBank> {
        match &self.ram {
            Some(ram) => {
                // clone the RAM
                let mut ram_banks = Vec::with_capacity(1);
                ram_banks.push(ram.clone());
                ram_banks
            }

            None => Vec::new()
        }
    }

    fn has_battery(&self) -> bool {
        self.header.cartridge_type == CartridgeType::RomRamBattery
    }
}