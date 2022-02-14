use std::convert;

#[derive(Clone, Copy)]
pub struct InterruptRegister {
    v_blank: bool,
    lcd_stat: bool,
    timer: bool,
    serial: bool,
    joypad: bool,
}

impl convert::From<InterruptRegister> for u8 {
    fn from(register: InterruptRegister) -> u8 {
        (if register.v_blank { 1 } else { 0 })
            | (if register.lcd_stat { 1 } else { 0 } << 1)
            | (if register.timer { 1 } else { 0 } << 2)
            | (if register.serial { 1 } else { 0 } << 3)
            | (if register.joypad { 1 } else { 0 } << 4)
    }
}

impl convert::From<u8> for InterruptRegister {
    fn from(byte: u8) -> Self {
        let v_blank = ((byte >> 0) & 0b1) != 0;
        let lcd_stat = ((byte >> 1) & 0b1) != 0;
        let timer = ((byte >> 2) & 0b1) != 0;
        let serial = ((byte >> 3) & 0b1) != 0;
        let joypad = ((byte >> 4) & 0b1) != 0;

        InterruptRegister {
            v_blank,
            lcd_stat,
            timer,
            serial,
            joypad,
        }
    }
}
