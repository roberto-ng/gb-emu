pub mod rom_only;

use std::convert::TryFrom;

use crate::{Result, EmulationError};
use num_enum::{TryFromPrimitive, IntoPrimitive};

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
    rom_size: u32,
    rom_banks: u16,
    ram_size: u32,
    ram_banks: u16,
}

pub trait Cartridge {
    fn read_byte_rom(&self, address: usize) -> Result<u8>;
    fn write_byte_rom(&mut self, address: usize, value: u8) -> Result<()>;

    fn read_byte_external_ram(&self, address: usize) -> Result<u8>;
    fn write_byte_external_ram(&mut self, address: usize, value: u8) -> Result<()>;
}

pub fn read_rom_header(rom: &Vec<u8>) -> Result<Header> {
    if rom.len() < 0x014F {
        return Err(EmulationError::InvalidRom)
    }

    let title = &rom[0x0134 ..= 0x0143];
    let title = read_rom_title(title);
    let cartridge_type = decode_cartridge_type(rom[0x0147])?;
    let (rom_size, rom_banks) = get_rom_size(rom[0x0148])?;
    let (ram_size, ram_banks) = get_ram_size(rom[0x0149])?;

    Ok(
        Header {
            title,
            cartridge_type,
            rom_size,
            rom_banks,
            ram_size,
            ram_banks,
        }
    )
}

fn read_rom_title(title_buffer: &[u8]) -> Option<String> {
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

const fn get_rom_size(code: u8) -> Result<(u32, u16)> {
    match code {
        0x00 => Ok((32_000, 2)),      // 32 KByte,  2 banks
        0x01 => Ok((64_000, 4)),      // 64 KByte,  4 banks
        0x02 => Ok((128_000, 8)),     // 128 KByte, 8 banks
        0x03 => Ok((256_000, 16)),    // 256 KByte, 16 banks
        0x04 => Ok((512_000, 32)),    // 512 KByte, 32 banks
        0x05 => Ok((1_000_000, 64)),  // 1 MByte,   64 banks
        0x06 => Ok((2_000_000, 128)), // 2 MByte,   128 banks
        0x07 => Ok((4_000_000, 256)), // 4 MByte,   256 banks
        0x08 => Ok((8_000_000, 512)), // 8 MByte,   512 banks
        _ => Err(EmulationError::InvalidRomSizeCode { code }),
    }
}

const fn get_ram_size(code: u8) -> Result<(u32, u16)> {
    match code {
        0x00 => Ok((0, 0)),        // No RAM
        0x01 => Ok((0, 0)),        // Unused
        0x02 => Ok((8_000, 1)),    // 8 KB, 1 bank
        0x03 => Ok((32_000, 4)),   // 32 KB, 4 banks of 8 KB each
        0x04 => Ok((128_000, 16)), // 128 KB, 16 banks of 8 KB each
        0x05 => Ok((64_000, 8)),   // 64 KB8 banks of 8 KB each
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