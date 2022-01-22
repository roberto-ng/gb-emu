use crate::{EmulationError, Result};
use crate::gpu::*;
use crate::cartridge::*;
use crate::cartridge::rom_only::*;

pub struct MemoryBus {
    gpu: Gpu,
    cartridge: Box<dyn Cartridge>,
    work_ram_0: [u8; WORK_RAM_0_SIZE],
    work_ram_1: [u8; WORK_RAM_N_SIZE],
    high_ram: [u8; HIGH_RAM_SIZE],
    ie: u8,
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        MemoryBus {
            gpu: Gpu::new(),
            cartridge: Box::new(RomOnlyCartridge::new()),
            work_ram_0: [0; WORK_RAM_0_SIZE],
            work_ram_1: [0; WORK_RAM_N_SIZE],
            high_ram: [0; HIGH_RAM_SIZE],
            ie: 0,
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

            ECHO_RAM_START ..= ECHO_RAM_END => {
                // TODO: Implement echo ram
                Ok(0)
            }

            OAM_BEGIN ..= OAM_END => {
                self.gpu.read_byte_oam(address)
            }

            IO_REGISTERS_START ..= IO_REGISTERS_END => {
                // TODO: Implement I/O registers
                Ok(0)
            }

            HIGH_RAM_START ..= HIGH_RAM_END => {
                let pos = address - HIGH_RAM_START;
                Ok(self.high_ram[pos])
            }

            INTERRUPT_ENABLE_REGISTER => {
                Ok(self.ie)
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

            EXTERNAL_RAM_START ..= EXTERNAL_RAM_START => {
                self.cartridge.write_byte_external_ram(address, value)
            }

            WORK_RAM_0_START ..= WORK_RAM_0_END => {
                let pos = address - WORK_RAM_0_START;
                self.work_ram_0[pos] = value;
                Ok(())
            }

            WORK_RAM_N_START ..= WORK_RAM_N_END => {
                let pos = address - WORK_RAM_N_START;
                self.work_ram_1[pos] = value;
                Ok(())
            }

            ECHO_RAM_START ..= ECHO_RAM_END => {
                // TODO: Implement echo ram
                Ok(())
            }

            OAM_BEGIN ..= OAM_END => {
                self.gpu.write_byte_oam(address, value)
            }

            IO_REGISTERS_START ..= IO_REGISTERS_END => {
                // TODO: Implement I/O registers
                Ok(())
            }

            HIGH_RAM_START ..= HIGH_RAM_END => {
                let pos = address - HIGH_RAM_START;
                self.high_ram[pos] = value;
                
                Ok(())
            }

            INTERRUPT_ENABLE_REGISTER => {
                self.ie = value;
                
                Ok(())
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

pub const ECHO_RAM_START: usize = 0xE000;
pub const ECHO_RAM_END: usize = 0xFDFF;
pub const ECHO_RAM_SIZE: usize = ECHO_RAM_END - ECHO_RAM_START + 1;

pub const IO_REGISTERS_START: usize = 0xFF00;
pub const IO_REGISTERS_END: usize = 0xFF7F;
pub const IO_REGISTERS_SIZE: usize = IO_REGISTERS_END - IO_REGISTERS_START + 1;

pub const HIGH_RAM_START: usize = 0xFF80;
pub const HIGH_RAM_END: usize = 0xFFFE;
pub const HIGH_RAM_SIZE: usize = HIGH_RAM_END - HIGH_RAM_START + 1;

pub const INTERRUPT_ENABLE_REGISTER: usize = 0xFFFF;