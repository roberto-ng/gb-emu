pub mod cartridge;
pub mod cpu;
pub mod cpu_registers;
pub mod error;
pub mod gpu;
pub mod instruction;
pub mod interrupt;
pub mod memory_bus;
pub mod timer;

use cartridge::create_cartridge;
use cpu::Cpu;
use error::Result;

pub struct GameBoy {
    pub cpu: Cpu,
}

impl GameBoy {
    pub fn new() -> GameBoy {
        GameBoy { cpu: Cpu::new() }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) -> Result<()> {
        // TODO: Reset everything before loading ROM
        let cartridge = create_cartridge(rom)?;
        self.cpu.bus.cartridge = Some(Box::new(cartridge));
        Ok(())
    }

    pub fn has_rom_loaded(&self) -> bool {
        match self.cpu.bus.cartridge {
            Some(_) => true,
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
