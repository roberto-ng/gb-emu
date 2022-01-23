pub mod rom_only;

use std::convert::TryFrom;

use crate::{Result, EmulationError};
use num_enum::{TryFromPrimitive, IntoPrimitive};

// each RAM bank has KiB of RAM
type RamBank = [u8; 8_192];

#[repr(u8)]
#[derive(Clone, Copy, TryFromPrimitive, IntoPrimitive)]
pub enum CartridgeType {
    RomOnly = 0x00,
    Mbc1 = 0x01,
    Mbc1Ram = 0x02,
    Mbc1RamBattery = 0x03,
    Mbc2 = 0x05,
    Mbc2Battery = 0x06,
    RomRam = 0x08,
    RomRamBattery = 0x09,
    Mmm01 = 0x0B,
    Mmm01Ram = 0x0C,
    Mmm01RamBattery = 0x0D,
    Mbc3TimerBattery = 0x0F,
    Mbc3TimerRamBattery = 0x10,
    Mbc3 = 0x11,
    Mbc3Ram = 0x12,
    Mbc3RamBattery = 0x13,
    Mbc5 = 0x19,
    Mbc5Ram = 0x1A,
    Mbc5RamBattery = 0x1B,
    Mbc5Rumble = 0x1C,
    Mbc5RumbleRam = 0x1D,
    Mbc5RumbleRamBattery = 0x1E,
    Mbc6 = 0x20,
    Mbc7SensorRumbleRamBattery = 0x22,
    PocketCamera = 0xFC,
    BandaiTama5 = 0xFD,
    HuC3 = 0xFE,
    HuC1RamBattery = 0xFF,
}

#[derive(Clone)]
pub struct Header {
    title: Option<String>,
    cartridge_type: CartridgeType,
    rom_size: usize,
    rom_bank_amount: usize,
    ram_size: usize,
    ram_bank_amount: usize,
}

pub trait Cartridge {
    fn read_byte_rom(&self, address: usize) -> Result<u8>;
    fn write_byte_rom(&mut self, address: usize, value: u8) -> Result<()>;

    fn read_byte_external_ram(&self, address: usize) -> Result<u8>;
    fn write_byte_external_ram(&mut self, address: usize, value: u8) -> Result<()>;

    fn get_rom_title(&self) -> Option<String>;
    fn get_ram_banks(&self) -> Vec<RamBank>;
}

pub fn read_rom_header(rom: &Vec<u8>) -> Result<Header> {
    if rom.len() < 0x014F {
        return Err(EmulationError::InvalidRom)
    }

    let title = &rom[0x0134 ..= 0x0143];
    let title = decode_rom_title(title);
    let cartridge_type = decode_cartridge_type(rom[0x0147])?;
    let (rom_size, rom_bank_amount) = get_rom_size(rom[0x0148])?;
    let (ram_size, ram_bank_amount) = get_ram_size(rom[0x0149])?;

    Ok(
        Header {
            title,
            cartridge_type,
            rom_size,
            rom_bank_amount,
            ram_size,
            ram_bank_amount,
        }
    )
}

fn decode_rom_title(title_buffer: &[u8]) -> Option<String> {
    match String::from_utf8(title_buffer.to_vec()) {
        Ok(title) => Some(title),
        Err(_) => None,
    }
}

fn decode_cartridge_type(code: u8) -> Result<CartridgeType> {
    if let Ok(cartridge_type) = CartridgeType::try_from(code) {
        Ok(cartridge_type)
    } else {
        Err(EmulationError::UnknownCartridgeType { code })
    }
}

const fn get_rom_size(code: u8) -> Result<(usize, usize)> {
    match code {
        0x00 => Ok((32_768, 2)),      // 32 KiB,  2 banks
        0x01 => Ok((65_536, 4)),      // 64 KiB,  4 banks
        0x02 => Ok((131_072, 8)),     // 128 KiB, 8 banks
        0x03 => Ok((262_144, 16)),    // 256 KiB, 16 banks
        0x04 => Ok((524_288, 32)),    // 512 KiB, 32 banks
        0x05 => Ok((1_048_576, 64)),  // 1 KiB,   64 banks
        0x06 => Ok((2_097_152, 128)), // 2 KiB,   128 banks
        0x07 => Ok((4_194_304, 256)), // 4 KiB,   256 banks
        0x08 => Ok((8_388_608, 512)), // 8 KiB,   512 banks
        _ => Err(EmulationError::InvalidRomSizeCode { code }),
    }
}

const fn get_ram_size(code: u8) -> Result<(usize, usize)> {
    match code {
        0x00 => Ok((0, 0)),        // No RAM
        0x01 => Ok((0, 0)),        // Unused
        0x02 => Ok((8_192, 1)),    // 8 KiB, 1 bank
        0x03 => Ok((32_768, 4)),   // 32 KiB, 4 banks of 8 KiB each
        0x04 => Ok((131_072, 16)), // 128 KiB, 16 banks of 8 KiB each
        0x05 => Ok((65_536, 8)),   // 64 KiB, 8 banks of 8 KiB each
        _ => Err(EmulationError::InvalidRomSizeCode { code }),
    }
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