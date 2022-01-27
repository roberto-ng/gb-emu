#[derive(Copy, Clone, Debug)]
pub struct Data {
    pub bytes: u16,
    pub cycles: u8,
    // The duration of conditional calls and returns is depending on if the action is taken or not
    pub action_cycles: Option<u8>,
    pub opcode: u8,
    pub is_prefixed: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    ADD(R, Data),
    JP(JumpTest, WordSource, Data),
    LD(LoadType, Data),
    PUSH(RR, Data),
    POP(RR, Data),
    CALL(JumpTest, Data),
    RET(JumpTest, Data),
    NOP(Data),
    Halt(Data),
}

// 8-bit registers
#[derive(Copy, Clone, Debug)]
pub enum R {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

// 16-bit registers
#[derive(Copy, Clone, Debug)]
pub enum RR {
    AF,
    BC,
    DE,
    HL,
}

#[derive(Copy, Clone, Debug)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

#[derive(Copy, Clone, Debug)]
pub enum ByteTarget {
    Register(R),
    Immediate8,
    HL,
    HLI,
    HLD,
    FF00PlusC,
}

#[derive(Copy, Clone, Debug)]
pub enum ByteSource {
    Register(R),
    Immediate8,
    HL,
    HLI,
    HLD,
    FF00PlusC,
}

#[derive(Copy, Clone, Debug)]
pub enum WordSource {
    SP,
    Immediate16,
    HL,
    SpPlusI8,
}

#[derive(Copy, Clone, Debug)]
pub enum WordTarget {
    SP,
    Direct,
    HL,
}

#[derive(Copy, Clone, Debug)]
pub enum LoadType {
    Byte(ByteTarget, ByteSource),
    Word(WordTarget, WordSource),
}


impl Data {
    pub const fn new(bytes: u16, cycles: u8, action_cycles: Option<u8>, opcode: u8) -> Data {
        Data {
            bytes,
            cycles,
            action_cycles,
            opcode,
            is_prefixed: false,
        }
    }

    pub fn get_action_cycles(&self) -> u8 {
        match self.action_cycles {
            Some(action_cycles) => action_cycles,
            None => {
                panic!(
                    "The instruction with opcode {:#06X} has no data about its amount of action cycles but \
                    it's still trying to use this data. This shouldn't happen.\
                    \n{:?}",
                    self.opcode,
                    self,
                );
            },
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
                Instruction::ADD(
                    R::B,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x81 => Some(
                Instruction::ADD(
                    R::C,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x82 => Some(
                Instruction::ADD(
                    R::D,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x83 => Some(
                Instruction::ADD(
                    R::E,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x84 => Some(
                Instruction::ADD(
                    R::H,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x85 => Some(
                Instruction::ADD(
                    R::L,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            // JP nz, u16
            0xC2 => Some(
                Instruction::JP(
                    JumpTest::NotZero,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), byte),
                )
            ),
            
            // JP n16
            0xC3 => Some(
                Instruction::JP(
                    JumpTest::Always,
                    WordSource::Immediate16,
                    Data::new(3, 16, None, byte),
                )
            ),
            
            // JP z, n16
            0xCA => Some(
                Instruction::JP(
                    JumpTest::Zero,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), byte),
                )
            ),

            // JP nc, n16
            0xD2 => Some(
                Instruction::JP(
                    JumpTest::NotCarry,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), byte),
                )
            ),

            // JP cc, n16
            0xDA => Some(
                Instruction::JP(
                    JumpTest::Carry,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), byte),
                )
            ),

            // JP HL
            0xE9 => Some(
                Instruction::JP(
                    JumpTest::Always,
                    WordSource::HL,
                    Data::new(1,4, None, byte),
                )
            ),

            _ => None,
        }
    }
}
