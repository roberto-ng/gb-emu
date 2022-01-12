pub struct Info {
    pub bytes: u8,
    pub cycles: u8,
    // The duration of conditional calls and returns is different when action is taken or not
    pub action_cycles: Option<u8>,
}

pub enum Instruction {
    ADD {
        target: ArithmeticTarget,
        info: Info,
    },
    JP {
        test: JumpTest,
        info: Info,
    },
}

pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl Info {
    const fn new(bytes: u8, cycles: u8, action_cycles: Option<u8>) -> Info {
        Info {
            bytes,
            cycles,
            action_cycles,
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
                Instruction::ADD {
                    target: ArithmeticTarget::B,
                    info: Info::new(1, 4, None),
                }
            ),
            0x81 => Some(
                Instruction::ADD {
                    target: ArithmeticTarget::C,
                    info: Info::new(1, 4, None),
                }
            ),
            0x82 => Some(
                Instruction::ADD {
                    target: ArithmeticTarget::D,
                    info: Info::new(1, 4, None),
                }
            ),
            0x83 => Some(
                Instruction::ADD {
                    target: ArithmeticTarget::E,
                    info: Info::new(1, 4, None),
                }
            ),
            0x84 => Some(
                Instruction::ADD {
                    target: ArithmeticTarget::H,
                    info: Info::new(1, 4, None),
                }
            ),
            0x85 => Some(
                Instruction::ADD {
                    target: ArithmeticTarget::L,
                    info: Info::new(1, 4, None),
                }
            ),
            0xC2 => Some(
                Instruction::JP {
                    test: JumpTest::NotZero,
                    info: Info::new(3, 12, Some(16)),
                }
            ),
            0xC3 => Some(
                Instruction::JP {
                    test: JumpTest::Always,
                    info: Info::new(3, 16, None),
                }
            ),
            0xCA => Some(
                Instruction::JP {
                    test: JumpTest::Zero,
                    info: Info::new(3, 12, Some(16)),
                }
            ),
            0xD2 => Some(
                Instruction::JP {
                    test: JumpTest::NotCarry,
                    info: Info::new(3, 12, Some(16)),
                }
            ),
            0xDA => Some(
                Instruction::JP {
                    test: JumpTest::Carry,
                    // cycles: 16/12
                    info: Info::new(3, 12, Some(16)),
                }
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
