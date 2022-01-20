use crate::gpu::*;

pub struct MemoryBus {
    memory: [u8; 0xFFFF],
    gpu: Gpu,
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            memory: [0; 0xFFFF],
            gpu: Gpu::new(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            VRAM_BEGIN ..= VRAM_END => {
                self.gpu.read_vram(address)
            }

            _ => {
                self.memory[address]
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            VRAM_BEGIN ..= VRAM_END => {
                self.gpu.write_vram(address, value);
            }
            
            _ => {
                self.memory[address] = value;
            }
        }
    }
}