use core::panic;
use std::convert;

use crate::interrupt::InterruptRegister;

pub struct Timers {
    /// FF04 - DIV - Divider Register (R/W)
    pub divider_register: u8,
    /// FF05 - TIMA - Timer counter (R/W)
    pub timer_counter: u8,
    // FF06 - TMA - Timer Modulo (R/W)
    pub timer_modulo: u8,
    /// FF07 - TAC - Timer Control (R/W)
    pub timer_control: TimerControl,
    /// FF0F - IF - Interrupt Flag (R/W)
    pub interrupt_flag_register: InterruptRegister,
    /// FFFF - IE - Interrupt Enable (R/W)
    pub interrupt_enable_register: InterruptRegister,
    /// Cycle counter in t-cycles
    cycle_count: u16,
}

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

impl Timers {
    pub fn new() -> Timers {
        Timers {
            divider_register: 0,
            timer_counter: 0,
            timer_modulo: 0,
            timer_control: 0.into(),
            interrupt_flag_register: 0.into(),
            interrupt_enable_register: 0.into(),
            cycle_count: 0,
        }
    }

    pub fn run(&mut self, cycles: u16) {
        let old_cycle_count = self.cycle_count;
        self.cycle_count = self.cycle_count.wrapping_add(cycles);

        let diff = self.cycle_count / 64 - old_cycle_count / 64;
        self.divider_register = self.divider_register.wrapping_add(diff as u8);

        // Check if the timer is enabled
        if self.timer_control.enable {
            let frequency = self.timer_control.speed.to_u16();
            let diff = self.cycle_count / frequency - old_cycle_count / frequency;
            
            let (next_timer_counter, did_overflow) = self.timer_counter.overflowing_add(diff as u8);
            self.timer_counter = next_timer_counter;
            if did_overflow {
                self.timer_counter = self.timer_modulo;
                // TODO: Request interrupt
            }
        }
    }
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

        TimerControl { enable, speed }
    }
}
