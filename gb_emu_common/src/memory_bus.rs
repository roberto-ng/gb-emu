use crate::cartridge::*;
use crate::error::{EmulationError, Result};
use crate::gpu::*;
use crate::timer::Timers;

pub struct MemoryBus {
    pub gpu: Gpu,
    pub cartridge: Option<Box<dyn Cartridge>>,
    work_ram_0: [u8; WORK_RAM_0_SIZE],
    work_ram_1: [u8; WORK_RAM_N_SIZE],
    high_ram: [u8; HIGH_RAM_SIZE],
    timers: Timers,
    sb: u8, // FF01 - SB - Serial transfer data (R/W)
}

impl MemoryBus {
    pub fn new() -> MemoryBus {
        //let cartridge = Box::new(create_cartridge(rom)?);

        MemoryBus {
            gpu: Gpu::new(),
            cartridge: None,
            work_ram_0: [0; WORK_RAM_0_SIZE],
            work_ram_1: [0; WORK_RAM_N_SIZE],
            high_ram: [0; HIGH_RAM_SIZE],
            timers: Timers::new(),
            sb: 0,
        }
    }

    pub fn read_byte(&self, address: u16) -> Result<u8> {
        let address = address as usize;
        // Return an error if we don't have a ROM loaded
        let cartridge = self.cartridge.as_ref().ok_or(EmulationError::NoRom)?;

        match address {
            ROM_BANK_0_START..=ROM_BANK_N_END => cartridge.read_byte_rom(address),

            VRAM_BEGIN..=VRAM_END => self.gpu.read_byte_vram(address),

            EXTERNAL_RAM_START..=EXTERNAL_RAM_END => cartridge.read_byte_external_ram(address),

            WORK_RAM_0_START..=WORK_RAM_0_END => {
                let pos = address - WORK_RAM_0_START;
                Ok(self.work_ram_0[pos])
            }

            WORK_RAM_N_START..=WORK_RAM_N_END => {
                let pos = address - WORK_RAM_N_START;
                Ok(self.work_ram_1[pos])
            }

            ECHO_RAM_START..=ECHO_RAM_END => {
                // use of this area is prohibited by Nintendo, so we won't implement it
                Ok(0)
            }

            OAM_BEGIN..=OAM_END => self.gpu.read_byte_oam(address),

            0xFEA0..=0xFEFF => Ok(0),

            INTERRUPT_ENABLE_REGISTER => {
                let register = self.timers.interrupt_enable_register.into();
                Ok(register)
            }

            INTERRUPT_FLAG_REGISTER => {
                let register = self.timers.interrupt_flag_register.into();
                Ok(register)
            }

            DIVIDER_REGISTER => Ok(self.timers.divider_register),

            TIMER_MODULO_REGISTER => Ok(self.timers.timer_modulo),

            0xFF44 => Ok(self.gpu.ly),

            IO_REGISTERS_START..=IO_REGISTERS_END => {
                // TODO: Implement I/O registers
                match address {
                    0xFF01 => Ok(self.sb),
                    _ => Ok(0),
                }
            }

            HIGH_RAM_START..=HIGH_RAM_END => {
                let pos = address - HIGH_RAM_START;
                Ok(self.high_ram[pos])
            }

            _ => {
                let error = EmulationError::InvalidMemoryRead { address };
                Err(error)
            }
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) -> Result<()> {
        if cfg!(not(target_family = "wasm")) {
            if address == 0xFF02 && value == 0x81 {
                let char = self.read_byte(0xFF01)?;
                let raw_text = vec![char];
                match String::from_utf8(raw_text) {
                    Ok(text) => print!("{text}"),
                    Err(_) => print!("<?>"),
                }
            }
        }

        let address = address as usize;
        // Return an error if we don't have a ROM loaded
        let cartridge = self.cartridge.as_mut().ok_or(EmulationError::NoRom)?;

        match address {
            ROM_BANK_0_START..=ROM_BANK_N_END => cartridge.write_byte_rom(address, value),

            VRAM_BEGIN..=VRAM_END => self.gpu.write_byte_vram(address, value),

            EXTERNAL_RAM_START..=EXTERNAL_RAM_START => {
                cartridge.write_byte_external_ram(address, value)
            }

            WORK_RAM_0_START..=WORK_RAM_0_END => {
                let pos = address - WORK_RAM_0_START;
                self.work_ram_0[pos] = value;
                Ok(())
            }

            WORK_RAM_N_START..=WORK_RAM_N_END => {
                let pos = address - WORK_RAM_N_START;
                self.work_ram_1[pos] = value;
                Ok(())
            }

            ECHO_RAM_START..=ECHO_RAM_END => {
                // use of this area is prohibited by Nintendo, so we won't implement it
                Ok(())
            }

            OAM_BEGIN..=OAM_END => self.gpu.write_byte_oam(address, value),

            0xFEA0..=0xFEFF => Ok(()),

            INTERRUPT_ENABLE_REGISTER => {
                self.timers.interrupt_enable_register = value.into();

                Ok(())
            }

            INTERRUPT_FLAG_REGISTER => {
                self.timers.interrupt_flag_register = value.into();

                Ok(())
            }

            DIVIDER_REGISTER => {
                // Writing any value to this register resets it to $00
                self.reset_divider_register();
                Ok(())
            }

            TIMER_MODULO_REGISTER => {
                self.timers.timer_modulo = value;
                Ok(())
            }

            IO_REGISTERS_START..=IO_REGISTERS_END => {
                // TODO: Implement I/O registers
                match address {
                    0xFF01 => {
                        self.sb = value;
                        Ok(())
                    },
                    _ => Ok(()),
                }
            }

            HIGH_RAM_START..=HIGH_RAM_END => {
                let pos = address - HIGH_RAM_START;
                self.high_ram[pos] = value;

                Ok(())
            }

            _ => {
                let error = EmulationError::InvalidMemoryWrite { address, value };
                Err(error)
            }
        }
    }

    pub fn reset_divider_register(&mut self) {
        self.timers.divider_register = 0;
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
pub const INTERRUPT_FLAG_REGISTER: usize = 0xFF0F;

// Timers
pub const DIVIDER_REGISTER: usize = 0xFF04;
pub const TIMER_COUNTER_REGISTER: usize = 0xFF05;
pub const TIMER_MODULO_REGISTER: usize = 0xFF06;
pub const TIMER_CONTROL_REGISTER: usize = 0xFF07;
