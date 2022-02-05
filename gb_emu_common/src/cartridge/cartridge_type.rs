use std::fmt::{self};

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq)]
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

impl fmt::Display for CartridgeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match &self {
            Self::RomOnly => "ROM only",
            Self::Mbc1 => "MBC1",
            Self::Mbc1Ram => "MBC1 + RAM",
            Self::Mbc1RamBattery => "MBC1 + RAM + battery",
            Self::Mbc2 => "MBC2",
            Self::Mbc2Battery => "MBC2 + battery",
            Self::RomRam => "ROM + RAM",
            Self::RomRamBattery => "ROM + RAM + battery",
            Self::Mmm01 => "MMM1",
            Self::Mmm01Ram => "MMM1 + RAM",
            Self::Mmm01RamBattery => "MMM1 + RAM + battery",
            Self::Mbc3TimerBattery => "MBC3 + timer + battery",
            Self::Mbc3TimerRamBattery => "MBC3 + timer + RAM + battery",
            Self::Mbc3 => "MBC3",
            Self::Mbc3Ram => "MBC3 + RAM",
            Self::Mbc3RamBattery => "MBC3 + RAM + battery",
            Self::Mbc5 => "MBC5",
            Self::Mbc5Ram => "MBC5 + RAM",
            Self::Mbc5RamBattery => "MBC5 + RAM + battery",
            Self::Mbc5Rumble => "MBC5 + rumble",
            Self::Mbc5RumbleRam => "MBC5 + rumble + RAM",
            Self::Mbc5RumbleRamBattery => "MBC5 + rumble + RAM + battery",
            Self::Mbc6 => "MBC6",
            Self::Mbc7SensorRumbleRamBattery => "MBC7 + sensor + rumble + RAM + battery",
            Self::PocketCamera => "Pocket Camera",
            Self::BandaiTama5 => "Bandai Tama 5",
            Self::HuC3 => "HuC3",
            Self::HuC1RamBattery => "HuC1 + RAM + battery",
        };

        write!(f, "{name}")
    }
}
