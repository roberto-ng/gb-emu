pub mod rom_only;

use crate::Result;

pub trait Cartridge {
    fn read_byte_rom(&self, address: usize) -> Result<u8>;
    fn write_byte_rom(&mut self, address: usize, value: u8) -> Result<()>;

    fn read_byte_external_ram(&self, address: usize) -> Result<u8>;
    fn write_byte_external_ram(&mut self, address: usize, value: u8) -> Result<()>;
}

pub const ROM_BANK_0_START: usize = 0x0000;
pub const ROM_BANK_0_END: usize = 0x3FFF;
pub const ROM_BANK_0_SIZE: usize = ROM_BANK_0_END - ROM_BANK_0_START + 1;

pub const ROM_BANK_N_START: usize = 0x4000;
pub const ROM_BANK_N_END: usize = 0x7FFF;
pub const ROM_BANK_N_SIZE: usize = ROM_BANK_N_END - ROM_BANK_N_START + 1;

pub const EXTERNAL_RAM_START: usize = 0xA000;
pub const EXTERNAL_RAM_END: usize = 0xBFFF;
pub const EXTERNAL_RAM_SIZE: usize = EXTERNAL_RAM_END - EXTERNAL_RAM_START + 1;