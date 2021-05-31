pub enum Instruction {
    ADD {
        target: ArithmeticTarget,
        info: InstructionInfo,
    },
    JP {
        test: JumpTest,
        info: InstructionInfo,
    },
}

pub struct InstructionInfo {
    pub bytes: u8,
    pub cycles: u8,
    // The duration of conditional calls and returns is different when action is taken or not
    pub action_cycles: Option<u8>,
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

impl InstructionInfo {
    const fn new(bytes: u8, cycles: u8, action_cycles: Option<u8>) -> InstructionInfo {
        InstructionInfo {
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
            0x80 => Some(Instruction::ADD {
                target: ArithmeticTarget::B,
                info: InstructionInfo::new(1, 4, None),
            }),
            0x81 => Some(Instruction::ADD {
                target: ArithmeticTarget::C,
                info: InstructionInfo::new(1, 4, None),
            }),
            0x82 => Some(Instruction::ADD {
                target: ArithmeticTarget::D,
                info: InstructionInfo::new(1, 4, None),
            }),
            0x83 => Some(Instruction::ADD {
                target: ArithmeticTarget::E,
                info: InstructionInfo::new(1, 4, None),
            }),
            0x84 => Some(Instruction::ADD {
                target: ArithmeticTarget::H,
                info: InstructionInfo::new(1, 4, None),
            }),
            0x85 => Some(Instruction::ADD {
                target: ArithmeticTarget::L,
                info: InstructionInfo::new(1, 4, None),
            }),
            0xC2 => Some(Instruction::JP {
                test: JumpTest::NotZero,
                // cycles: 16/12
                info: InstructionInfo::new(3, 12, Some(16)),
            }),
            0xC3 => Some(Instruction::JP {
                test: JumpTest::Always,
                info: InstructionInfo::new(3, 16, None),
            }),
            0xCA => Some(Instruction::JP {
                test: JumpTest::Zero,
                // cycles: 16/12
                info: InstructionInfo::new(3, 12, Some(16)),
            }),
            0xD2 => Some(Instruction::JP {
                test: JumpTest::NotCarry,
                info: InstructionInfo::new(3, 12, Some(16)),
            }),
            0xDA => Some(Instruction::JP {
                test: JumpTest::Carry,
                // cycles: 16/12
                info: InstructionInfo::new(3, 12, Some(16)),
            }),
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
