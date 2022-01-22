use crate::{EmulationError, Result};
use crate::gpu::*;
use crate::cartridge::*;
use crate::cartridge::rom_only::*;

pub struct MemoryBus {
    cartridge: Box<dyn Cartridge>,
    gpu: Gpu,
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            cartridge: Box::new(RomOnlyCartridge::new()),
            gpu: Gpu::new(),
        }
    }

    pub fn read_byte(&self, address: u16) -> Result<u8> {
        let address = address as usize;
        match address {
            ROM_BANK_0_START ..= ROM_BANK_N_END => {
                self.cartridge.read_byte(address)
            }

            VRAM_BEGIN ..= VRAM_END => {
                self.gpu.read_vram_byte(address)
            }

            OAM_BEGIN ..= OAM_END => {
                self.gpu.read_oam_byte(address)
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
                self.cartridge.write_byte(address, value)
            }

            VRAM_BEGIN ..= VRAM_END => {
                self.gpu.write_vram_byte(address, value)
            }

            OAM_BEGIN ..= OAM_END => {
                self.gpu.write_oam_byte(address, value)
            }
            
            _ => {
                let error = EmulationError::InvalidMemoryWrite { address, value };
                Err(error)
            }
        }
    }
}
