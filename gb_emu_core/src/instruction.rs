#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Data {
    pub bytes: u16,
    pub cycles: u8,
    // The duration of conditional calls and returns is different when action is taken or not
    pub action_cycles: Option<u8>,
    pub opcode: u8,
    pub is_prefixed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Instruction {
    ADD(ArithmeticTarget, Data),
    JP(JumpTest, Data),
    LD(LoadType, Data),
    PUSH(StackTarget, Data),
    POP(StackTarget, Data),
    CALL(JumpTest, Data),
    RET(JumpTest, Data),
    NOP(Data),
    Halt(Data),
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

#[derive(Copy, Clone, Debug, PartialEq)]

pub enum LoadByteTarget {
    A, B, C, D, E, H, L, HLI,
}

#[derive(Copy, Clone, Debug, PartialEq)]

pub enum LoadByteSource {
    A, B, C, D, E, H, L, D8, HLI,
}

#[derive(Copy, Clone, Debug, PartialEq)]

pub enum LoadType {
    Byte(LoadByteTarget, LoadByteSource)
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StackTarget {
    BC,
}

impl Data {
    const fn new(bytes: u16, cycles: u8, action_cycles: Option<u8>, opcode: u8) -> Data {
        Data {
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

    const fn from_byte_prefixed(_byte: u8) -> Option<Instruction> {
        None
    }

    const fn from_byte_not_prefixed(byte: u8) -> Option<Instruction> {
        match byte {
            0x80 => Some(
                Instruction::ADD (
                    ArithmeticTarget::B,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x81 => Some(
                Instruction::ADD (
                    ArithmeticTarget::C,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x82 => Some(
                Instruction::ADD (
                    ArithmeticTarget::D,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x83 => Some(
                Instruction::ADD (
                    ArithmeticTarget::E,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x84 => Some(
                Instruction::ADD (
                    ArithmeticTarget::H,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x85 => Some(
                Instruction::ADD (
                    ArithmeticTarget::L,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0xC2 => Some(
                Instruction::JP (
                    JumpTest::NotZero,
                    Data::new(3, 12, Some(16), byte),
                )
            ),
            
            0xC3 => Some(
                Instruction::JP (
                    JumpTest::Always,
                    Data::new(3, 16, None, byte),
                )
            ),
            
            0xCA => Some(
                Instruction::JP (
                    JumpTest::Zero,
                    Data::new(3, 12, Some(16), byte),
                )
            ),

            0xD2 => Some(
                Instruction::JP (
                    JumpTest::NotCarry,
                    Data::new(3, 12, Some(16), byte),
                )
            ),

            0xDA => Some(
                Instruction::JP (
                    JumpTest::Carry,
                    Data::new(3, 12, Some(16), byte),
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
