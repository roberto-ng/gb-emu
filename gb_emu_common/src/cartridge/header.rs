use std::convert::TryFrom;

use crate::{Result, EmulationError};
use crate::cartridge::cartridge_type::*;

#[derive(Clone)]
pub struct Header {
    pub title: Option<String>,
    pub cartridge_type: CartridgeType,
    pub rom_bank_amount: usize,
    pub ram_bank_amount: usize,
}

impl Header {
    pub fn read_rom_header(rom: &Vec<u8>) -> Result<Header> {
        if rom.len() < 0x014F {
            return Err(EmulationError::InvalidRom)
        }
    
        let title = &rom[0x0134 ..= 0x0143];
        let title = decode_rom_title(title);
        let cartridge_type = decode_cartridge_type(rom[0x0147])?;
        let rom_bank_amount = get_amount_of_rom_banks(rom[0x0148])?;
        let ram_bank_amount = get_amount_of_ram_banks(rom[0x0149])?;
    
        Ok(
            Header {
                title,
                cartridge_type,
                rom_bank_amount,
                ram_bank_amount,
            }
        )
    }    
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

const fn get_amount_of_rom_banks(code: u8) -> Result<usize> {
    match code {
        0x00 => Ok(2),   // 32 KiB,  2 banks
        0x01 => Ok(4),   // 64 KiB,  4 banks
        0x02 => Ok(8),   // 128 KiB, 8 banks
        0x03 => Ok(16),  // 256 KiB, 16 banks
        0x04 => Ok(32),  // 512 KiB, 32 banks
        0x05 => Ok(64),  // 1 KiB,   64 banks
        0x06 => Ok(128), // 2 KiB,   128 banks
        0x07 => Ok(256), // 4 KiB,   256 banks
        0x08 => Ok(512), // 8 KiB,   512 banks
        _ => Err(EmulationError::InvalidRomSizeCode { code }),
    }
}

const fn get_amount_of_ram_banks(code: u8) -> Result<usize> {
    match code {
        0x00 => Ok(0),  // No RAM
        0x01 => Ok(0),  // Unused
        0x02 => Ok(1),  // 8 KiB, 1 bank
        0x03 => Ok(4),  // 32 KiB, 4 banks of 8 KiB each
        0x04 => Ok(16), // 128 KiB, 16 banks of 8 KiB each
        0x05 => Ok(8),  // 64 KiB, 8 banks of 8 KiB each
        _ => Err(EmulationError::InvalidRomSizeCode { code }),
    }
}