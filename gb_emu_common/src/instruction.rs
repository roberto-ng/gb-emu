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
    Add(R, Data),
    Add16Bits(WordSource, WordTarget, Data),
    And(ByteSource, Data),
    Bit(u8, ByteSource, Data),
    CCF(Data),
    Cp(ByteSource, Data),
    Cpl(Data),
    DAA(Data),
    Dec(ByteTarget, Data),
    Dec16Bits(WordTarget, Data),
    DI(Data),
    EI(Data),
    Inc(ByteTarget, Data),
    Inc16Bits(WordTarget, Data),
    Or(ByteSource, Data),
    Res(u8, ByteTarget, Data),
    RL(ByteTarget, Data),
    RLC(ByteTarget, Data),
    RR(ByteTarget, Data),
    RRC(ByteTarget, Data),
    RST(u8, Data),
    SbC(ByteSource, Data),
    SCF(Data),
    Set(u8, ByteTarget, Data),
    SLA(ByteTarget, Data),
    SRA(ByteTarget, Data),
    SRL(ByteTarget, Data),
    Stop(Data),
    Sub(ByteSource, Data),
    Swap(ByteTarget, Data),
    XOr(ByteSource, Data),
    Jp(JumpTest, WordSource, Data),
    JR(JumpTest, Data),
    Ld(LoadType, Data),
    Push(RR, Data),
    Pop(RR, Data),
    Call(JumpTest, Data),
    Ret(JumpTest, Data),
    RetI(Data),
    NoOp(Data),
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
    Registers(RR),
    Immediate8,
    HL,
    HLI,
    HLD,
    FF00PlusC,
}

#[derive(Copy, Clone, Debug)]
pub enum ByteSource {
    Register(R),
    Registers(RR),
    Immediate8,
    HL,
    HLI,
    HLD,
    FF00PlusC,
}

#[derive(Copy, Clone, Debug)]
pub enum WordSource {
    Registers(RR),
    SP,
    Immediate16,
    SpPlusI8,
}

#[derive(Copy, Clone, Debug)]
pub enum WordTarget {
    Registers(RR),
    SP,
    Direct,
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
                    "\
                    The instruction with opcode {:#06X} has no data about its amount of action cycles but \
                    it's still trying to use this data. This shouldn't happen.\n\
                    {:?}\
                    ",
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
            0x00 => Some(
                Instruction::NoOp(
                    Data::new(1, 1, None, byte)
                )
            ),

            0x01 => Some(
                Instruction::Ld(
                    LoadType::Word(
                        WordTarget::Registers(RR::BC), 
                        WordSource::Immediate16
                    ), 
                    Data::new(3, 3, None, byte)
                )
            ),

            0x02 => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Registers(RR::BC),
                        ByteSource::Register(R::A)
                    ),
                    Data::new(1, 2, None, byte)
                )
            ),

            0x03 => Some(
                Instruction::Inc(
                    ByteTarget::Registers(RR::BC),
                    Data::new(1, 2, None, byte)
                )
            ),

            0x04 => Some(
                Instruction::Inc(
                    ByteTarget::Register(R::B),
                    Data::new(1, 1, None, byte)
                )
            ),

            0x05 => Some(
                Instruction::Dec(
                    ByteTarget::Register(R::B),
                    Data::new(1, 1, None, byte)
                )
            ),

            0x06 => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::B),
                        ByteSource::Immediate8,
                    ),
                    Data::new(2, 2, None, byte)
                )
            ),

            0x07 => Some(
                Instruction::RLC(
                    ByteTarget::Register(R::A),
                    Data::new(1, 1, None, byte)
                )
            ),

            0x08 => Some(
                Instruction::Ld(
                    LoadType::Word(
                        WordTarget::Direct,
                        WordSource::SP
                    ),
                    Data::new(3, 5, None, byte)
                )
            ),

            0x09 => Some(
                Instruction::Add16Bits(
                    WordSource::Registers(RR::BC),
                    WordTarget::Registers(RR::HL),
                    Data::new(1, 2, None, byte)
                ),
            ),

            0x0A => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::A),
                        ByteSource::Registers(RR::BC),
                    ),
                    Data::new(1, 2, None, byte)
                )
            ),

            0x0B => Some(
                Instruction::Dec16Bits(
                    WordTarget::Registers(RR::BC),
                    Data::new(1, 2, None, byte)
                )
            ),

            0x0C => Some(
                Instruction::Inc(
                    ByteTarget::Register(R::C),
                    Data::new(1, 1, None, byte)
                )
            ),

            0x0D => Some(
                Instruction::Dec(
                    ByteTarget::Register(R::C),
                    Data::new(1, 1, None, byte)
                )
            ),

            0x0E => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::C),
                        ByteSource::Immediate8,
                    ),
                    Data::new(2, 2, None, byte)
                )
            ),

            0x0F => Some(
                Instruction::RRC(
                    ByteTarget::Register(R::A),
                    Data::new(1, 1, None, byte)
                )
            ),

            0x10 => Some(
                Instruction::Stop(
                    Data::new(1, 1, None, byte)
                )
            ),

            0x11 => Some(
                Instruction::Ld(
                    LoadType::Word(
                        WordTarget::Registers(RR::DE),
                        WordSource::Immediate16,
                    ),
                    Data::new(3, 3, None, byte)
                )
            ),

            0x12 => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Registers(RR::DE),
                        ByteSource::Register(R::A),
                    ),
                    Data::new(1, 2, None, byte)
                )
            ),

            0x13 => Some(
                Instruction::Inc16Bits(
                    WordTarget::Registers(RR::DE),
                    Data::new(1, 2, None, byte)
                )
            ),

            0x14 => Some(
                Instruction::Inc(
                    ByteTarget::Registers(RR::HL),
                    Data::new(1, 3, None, byte)
                ),
            ),

            0x15 => Some(
                Instruction::Dec(
                    ByteTarget::Register(R::D),
                    Data::new(1, 1, None, byte)
                )
            ),

            0x16 => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::D),
                        ByteSource::Immediate8,
                    ),
                    Data::new(2, 2, None, byte)
                )
            ),

            0x80 => Some(
                Instruction::Add(
                    R::B,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x81 => Some(
                Instruction::Add(
                    R::C,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x82 => Some(
                Instruction::Add(
                    R::D,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x83 => Some(
                Instruction::Add(
                    R::E,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x84 => Some(
                Instruction::Add(
                    R::H,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            0x85 => Some(
                Instruction::Add(
                    R::L,
                    Data::new(1, 4, None, byte),
                )
            ),
            
            // JP nz, u16
            0xC2 => Some(
                Instruction::Jp(
                    JumpTest::NotZero,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), byte),
                )
            ),
            
            // JP n16
            0xC3 => Some(
                Instruction::Jp(
                    JumpTest::Always,
                    WordSource::Immediate16,
                    Data::new(3, 16, None, byte),
                )
            ),
            
            // JP z, n16
            0xCA => Some(
                Instruction::Jp(
                    JumpTest::Zero,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), byte),
                )
            ),

            // JP nc, n16
            0xD2 => Some(
                Instruction::Jp(
                    JumpTest::NotCarry,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), byte),
                )
            ),

            // JP cc, n16
            0xDA => Some(
                Instruction::Jp(
                    JumpTest::Carry,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), byte),
                )
            ),

            // JP HL
            0xE9 => Some(
                Instruction::Jp(
                    JumpTest::Always,
                    WordSource::Registers(RR::HL),
                    Data::new(1,4, None, byte),
                )
            ),

            _ => None,
        }
    }
}
