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

    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            ROM_BANK_0_START ..= ROM_BANK_N_END => {
                self.cartridge.read_byte(address)
            }

            VRAM_BEGIN ..= VRAM_END => {
                self.gpu.read_vram(address)
            }

            _ => {
                panic!("Invalid read on address {address:#06X}");
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            ROM_BANK_0_START ..= ROM_BANK_N_END => {
                self.cartridge.write_byte(address, value);
            }

            VRAM_BEGIN ..= VRAM_END => {
                self.gpu.write_vram(address, value);
            }
            
            _ => {
                panic!("Invalid write on address {address:#06X}");
            }
        }
    }
}
