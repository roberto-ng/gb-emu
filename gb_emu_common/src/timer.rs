use core::panic;
use std::convert;

pub struct TimerControl {
    enable: bool,
    speed: CpuSpeed,
}

pub enum CpuSpeed {
    Clock1024,
    Clock16,
    Clock64,
    Clock256,
}

impl TimerControl {
    pub fn new() -> TimerControl {
        0.into()
    }
}

impl CpuSpeed {
    fn to_u16(&self) -> u16 {
        match *self {
            CpuSpeed::Clock1024 => 1024,
            CpuSpeed::Clock16 => 16,
            CpuSpeed::Clock256 => 256,
            CpuSpeed::Clock64 => 64,
        }
    }
}

impl convert::From<TimerControl> for u8 {
    fn from(timer: TimerControl) -> u8 {
        let enable: u8 = if timer.enable { 1 } else { 0 };
        let speed: u8 = match timer.speed {
            CpuSpeed::Clock1024 => 0b00,
            CpuSpeed::Clock16 => 0b01,
            CpuSpeed::Clock64 => 0b10,
            CpuSpeed::Clock256 => 0b11,
        };

        (enable << 2) | speed
    }
}

impl convert::From<u8> for TimerControl {
    fn from(byte: u8) -> Self {
        let enable = (byte & 0b00000100) >> 2;
        let enable = if enable != 0 { true } else { false };

        let speed = byte & 0x00000011;
        let speed = match speed {
            0b00 => CpuSpeed::Clock1024,
            0b01 => CpuSpeed::Clock16,
            0b10 => CpuSpeed::Clock64,
            0b11 => CpuSpeed::Clock256,
            _ => panic!("This should be unreachable"),
        };

        TimerControl {
            enable,
            speed,
        }
    }
}