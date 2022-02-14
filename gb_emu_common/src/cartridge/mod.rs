pub mod cartridge_type;
pub mod header;
pub mod rom_only;

use self::cartridge_type::*;
use self::header::Header;
use self::rom_only::RomOnlyCartridge;
use crate::error::{EmulationError, Result};

// each RAM bank has KiB of RAM
type RamBank = [u8; RAM_BANK_SIZE];

pub trait Cartridge {
    fn read_byte_rom(&self, address: usize) -> Result<u8>;
    fn write_byte_rom(&mut self, address: usize, value: u8) -> Result<()>;

    fn read_byte_external_ram(&self, address: usize) -> Result<u8>;
    fn write_byte_external_ram(&mut self, address: usize, value: u8) -> Result<()>;

    fn get_header(&self) -> Header;
    fn get_ram_banks(&self) -> Vec<RamBank>;
    fn has_battery(&self) -> bool;
}

pub fn create_cartridge(rom: Vec<u8>) -> Result<impl Cartridge> {
    let header = Header::read_rom_header(&rom)?;
    match header.cartridge_type {
        CartridgeType::RomOnly | CartridgeType::RomRam | CartridgeType::RomRamBattery => {
            RomOnlyCartridge::new(rom, header)
        }

        _ => {
            let error = EmulationError::UnsupportedCartridgeType {
                cartridge_type: header.cartridge_type,
            };
            Err(error)
        }
    }
}

pub const ROM_BANK_SIZE: usize = 16_384;
pub const RAM_BANK_SIZE: usize = 8_192;

pub const ROM_BANK_0_START: usize = 0x0000;
pub const ROM_BANK_0_END: usize = 0x3FFF;
pub const ROM_BANK_0_SIZE: usize = ROM_BANK_0_END - ROM_BANK_0_START + 1;

pub const ROM_BANK_N_START: usize = 0x4000;
pub const ROM_BANK_N_END: usize = 0x7FFF;
pub const ROM_BANK_N_SIZE: usize = ROM_BANK_N_END - ROM_BANK_N_START + 1;

pub const EXTERNAL_RAM_START: usize = 0xA000;
pub const EXTERNAL_RAM_END: usize = 0xBFFF;
pub const EXTERNAL_RAM_SIZE: usize = EXTERNAL_RAM_END - EXTERNAL_RAM_START + 1;
