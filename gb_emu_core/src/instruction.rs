#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Info {
    pub bytes: u8,
    pub cycles: u8,
    // The duration of conditional calls and returns is different when action is taken or not
    pub action_cycles: Option<u8>,
    pub opcode: u8,
    pub is_prefixed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction {
    ADD (ArithmeticTarget, Info),
    JP (JumpTest, Info),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl Info {
    const fn new(bytes: u8, cycles: u8, action_cycles: Option<u8>, opcode: u8) -> Info {
        Info {
            bytes,
            cycles,
            action_cycles,
            opcode,
            is_prefixed: false,
        }
    }
}

impl Instruction {
    pub const fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Instruction::from_byte_prefixed(byte)
        } else {
            Instruction::from_byte_not_prefixed(byte)
        }
    }

    const fn from_byte_prefixed(byte: u8) -> Option<Instruction> {
        None
    }

    const fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            0x80 => Some(
                Instruction::ADD (
                    ArithmeticTarget::B,
                    Info::new(1, 4, None, byte),
                )
            ),
            0x81 => Some(
                Instruction::ADD (
                    ArithmeticTarget::C,
                    Info::new(1, 4, None, byte),
                )
            ),
            0x82 => Some(
                Instruction::ADD (
                    ArithmeticTarget::D,
                    Info::new(1, 4, None, byte),
                )
            ),
            0x83 => Some(
                Instruction::ADD (
                    ArithmeticTarget::E,
                    Info::new(1, 4, None, byte),
                )
            ),
            0x84 => Some(
                Instruction::ADD (
                    ArithmeticTarget::H,
                    Info::new(1, 4, None, byte),
                )
            ),
            0x85 => Some(
                Instruction::ADD (
                    ArithmeticTarget::L,
                    Info::new(1, 4, None, byte),
                )
            ),
            0xC2 => Some(
                Instruction::JP (
                    JumpTest::NotZero,
                    Info::new(3, 12, Some(16), byte),
                )
            ),
            0xC3 => Some(
                Instruction::JP (
                    JumpTest::Always,
                    Info::new(3, 16, None, byte),
                )
            ),
            0xCA => Some(
                Instruction::JP (
                    JumpTest::Zero,
                    Info::new(3, 12, Some(16), byte),
                )
            ),
            0xD2 => Some(
                Instruction::JP (
                    JumpTest::NotCarry,
                    Info::new(3, 12, Some(16), byte),
                )
            ),
            0xDA => Some(
                Instruction::JP (
                    JumpTest::Carry,
                    Info::new(3, 12, Some(16), byte),
                )
            ),
            /*
            0xE9 => Some(Instruction::JP {
                test: JumpTest::Always,
                source: //to hl?,
                info: InstructionInfo::new(1, 4, None),
            }),
            */
            _ => None,
        }
    }
}
