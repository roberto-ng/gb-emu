use crate::cartridge::*;

pub struct RomOnlyCartridge {
    rom_bank_0: [u8; ROM_BANK_0_SIZE],
    rom_bank_n: [u8; ROM_BANK_N_SIZE],
}

impl RomOnlyCartridge {
    pub fn new() -> RomOnlyCartridge {
        RomOnlyCartridge {
            rom_bank_0: [0; ROM_BANK_0_SIZE],
            rom_bank_n: [0; ROM_BANK_N_SIZE],
        }
    }
}

impl Cartridge for RomOnlyCartridge {
    fn read_byte(&self, address: usize) -> u8 {
        match address {
            ROM_BANK_0_START ..= ROM_BANK_0_END => {
                self.rom_bank_0[address]
            }

            ROM_BANK_N_START ..= ROM_BANK_N_END => {
                self.rom_bank_n[address]
            }

            _ => {
                panic!("Invalid cartridge read on address {address:#06X}");
            }
        }
    }

    fn write_byte(&self, _address: usize, _value: u8) {}    
}