use crate::{EmulationError, Result};
use crate::gpu::*;
use crate::cartridge::*;
use crate::cartridge::rom_only::*;

pub struct MemoryBus {
    gpu: Gpu,
    cartridge: Box<dyn Cartridge>,
    work_ram_0: [u8; WORK_RAM_0_SIZE],
    work_ram_1: [u8; WORK_RAM_N_SIZE],
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            gpu: Gpu::new(),
            cartridge: Box::new(RomOnlyCartridge::new()),
            work_ram_0: [0; WORK_RAM_0_SIZE],
            work_ram_1: [0; WORK_RAM_N_SIZE],
        }
    }

    pub fn read_byte(&self, address: u16) -> Result<u8> {
        let address = address as usize;
        match address {
            ROM_BANK_0_START ..= ROM_BANK_N_END => {
                self.cartridge.read_byte_rom(address)
            }

            VRAM_BEGIN ..= VRAM_END => {
                self.gpu.read_byte_vram(address)
            }

            OAM_BEGIN ..= OAM_END => {
                self.gpu.read_byte_oam(address)
            }

            EXTERNAL_RAM_START ..= EXTERNAL_RAM_END => {
                self.cartridge.read_byte_external_ram(address)
            }

            WORK_RAM_0_START ..= WORK_RAM_0_END => {
                let pos = address - WORK_RAM_0_START;
                Ok(self.work_ram_0[pos])
            }

            WORK_RAM_N_START ..= WORK_RAM_N_END => {
                let pos = address - WORK_RAM_N_START;
                Ok(self.work_ram_1[pos])
            }

            _ => {
                let error = EmulationError::InvalidMemoryRead { address };
                Err(error)
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) -> Result<()> {
        let address = address as usize;
        match address {
            ROM_BANK_0_START ..= ROM_BANK_N_END => {
                self.cartridge.write_byte_rom(address, value)
            }

            VRAM_BEGIN ..= VRAM_END => {
                self.gpu.write_byte_vram(address, value)
            }

            OAM_BEGIN ..= OAM_END => {
                self.gpu.write_byte_oam(address, value)
            }

            EXTERNAL_RAM_START ..= EXTERNAL_RAM_START => {
                self.cartridge.write_byte_external_ram(address, value)
            }
            
            _ => {
                let error = EmulationError::InvalidMemoryWrite { address, value };
                Err(error)
            }
        }
    }
}

pub const WORK_RAM_0_START: usize = 0xC000;
pub const WORK_RAM_0_END: usize = 0xCFFF;
pub const WORK_RAM_0_SIZE: usize = WORK_RAM_0_END - WORK_RAM_0_START + 1;

pub const WORK_RAM_N_START: usize = 0xD000;
pub const WORK_RAM_N_END: usize = 0xDFFF;
pub const WORK_RAM_N_SIZE: usize = WORK_RAM_N_END - WORK_RAM_N_START + 1;