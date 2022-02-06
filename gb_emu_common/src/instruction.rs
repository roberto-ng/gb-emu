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

    const fn from_byte_not_prefixed(opcode: u8) -> Option<Instruction> {
        match opcode {
            // NOP
            0x00 => Some(
                Instruction::NoOp(
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // LD BC, u16
            0x01 => Some(
                Instruction::Ld(
                    LoadType::Word(
                        WordTarget::Registers(RR::BC), 
                        WordSource::Immediate16
                    ), 
                    Data::new(3, 12, None, opcode),
                ),
            ),

            // LD (BC), A
            0x02 => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Registers(RR::BC),
                        ByteSource::Register(R::A),
                    ),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // INC BC
            0x03 => Some(
                Instruction::Inc(
                    ByteTarget::Registers(RR::BC),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // INC B
            0x04 => Some(
                Instruction::Inc(
                    ByteTarget::Register(R::B),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // DEC B
            0x05 => Some(
                Instruction::Dec(
                    ByteTarget::Register(R::B),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // LD B, u8
            0x06 => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::B),
                        ByteSource::Immediate8,
                    ),
                    Data::new(2, 8, None, opcode),
                ),
            ),

            // RLCA
            0x07 => Some(
                Instruction::RLC(
                    ByteTarget::Register(R::A),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // LD (u16), SP
            0x08 => Some(
                Instruction::Ld(
                    LoadType::Word(
                        WordTarget::Direct,
                        WordSource::SP,
                    ),
                    Data::new(3, 20, None, opcode),
                ),
            ),

            // ADD HL, BC
            0x09 => Some(
                Instruction::Add16Bits(
                    WordSource::Registers(RR::BC),
                    WordTarget::Registers(RR::HL),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // LD A, (BC)
            0x0A => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::A),
                        ByteSource::Registers(RR::BC),
                    ),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // DEC BC
            0x0B => Some(
                Instruction::Dec16Bits(
                    WordTarget::Registers(RR::BC),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // INC C
            0x0C => Some(
                Instruction::Inc(
                    ByteTarget::Register(R::C),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // DEC C
            0x0D => Some(
                Instruction::Dec(
                    ByteTarget::Register(R::C),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // LD C, u8
            0x0E => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::C),
                        ByteSource::Immediate8,
                    ),
                    Data::new(2, 8, None, opcode),
                ),
            ),

            // RRCA
            0x0F => Some(
                Instruction::RRC(
                    ByteTarget::Register(R::A),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // STOP
            0x10 => Some(
                Instruction::Stop(
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // LD DE, u16
            0x11 => Some(
                Instruction::Ld(
                    LoadType::Word(
                        WordTarget::Registers(RR::DE),
                        WordSource::Immediate16,
                    ),
                    Data::new(3, 12, None, opcode),
                ),
            ),

            // LD (DE),A
            0x12 => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Registers(RR::DE),
                        ByteSource::Register(R::A),
                    ),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // INC DE
            0x13 => Some(
                Instruction::Inc16Bits(
                    WordTarget::Registers(RR::DE),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // INC D
            0x14 => Some(
                Instruction::Inc(
                    ByteTarget::Register(R::D),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // DEC D
            0x15 => Some(
                Instruction::Dec(
                    ByteTarget::Register(R::D),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // LD D, u8
            0x16 => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::D),
                        ByteSource::Immediate8,
                    ),
                    Data::new(2, 8, None, opcode),
                ),
            ),

            // RLA
            0x17 => Some(
                Instruction::RL(
                    ByteTarget::Register(R::A),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // JR i8
            0x18 => Some(
                Instruction::JR(
                    JumpTest::Always,
                    Data::new(2, 12, Some(12), opcode),
                ),
            ),

            // ADD HL, DE
            0x19 => Some(
                Instruction::Add16Bits(
                    WordSource::Registers(RR::DE),
                    WordTarget::Registers(RR::HL),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // LD A, (DE)
            0x1A => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::A),
                        ByteSource::Registers(RR::DE),
                    ),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // DEC DE
            0x1B => Some(
                Instruction::Dec16Bits(
                    WordTarget::Registers(RR::DE),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // INC E
            0x1C => Some(
                Instruction::Inc(
                    ByteTarget::Register(R::E),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // DEC E
            0x1D => Some(
                Instruction::Dec(
                    ByteTarget::Register(R::E),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // LD E, u8
            0x1E => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::E),
                        ByteSource::Immediate8,
                    ),
                    Data::new(2, 8, None, opcode),
                ),
            ),

            // RRA
            0x1F => Some(
                Instruction::RR(
                    ByteTarget::Register(R::A),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // JR NZ, i8
            0x20 => Some(
                Instruction::JR(
                    JumpTest::NotZero,
                    Data::new(2, 8, Some(12), opcode),
                ),
            ),

            // LD HL, u16
            0x21 => Some(
                Instruction::Ld(
                    LoadType::Word(
                        WordTarget::Registers(RR::HL),
                        WordSource::Immediate16
                    ),
                    Data::new(3, 12, None, opcode),
                ),
            ),

            // LD (HL+),A
            0x22 => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::HLI,
                        ByteSource::Register(R::A),
                    ),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // INC HL
            0x23 => Some(
                Instruction::Inc16Bits(
                    WordTarget::Registers(RR::HL),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // INC H
            0x24 => Some(
                Instruction::Inc(
                    ByteTarget::Register(R::H),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // DEC H
            0x25 => Some(
                Instruction::Dec(
                    ByteTarget::Register(R::H),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // LD H, u8
            0x26 => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::H),
                        ByteSource::Immediate8,
                    ),
                    Data::new(2, 8, None, opcode),
                ),
            ),

            // DAA
            0x27 => Some(
                Instruction::DAA(
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // JR Z, i8
            0x28 => Some(
                Instruction::JR(
                    JumpTest::Zero,
                    Data::new(2, 8, Some(12), opcode),
                ),
            ),

            // ADD HL, HL
            0x29 => Some(
                Instruction::Add16Bits(
                    WordSource::Registers(RR::HL),
                    WordTarget::Registers(RR::HL),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // LD A, (HL+)
            0x2A => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::A),
                        ByteSource::HLI,
                    ),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // DEC HL
            0x2B => Some(
                Instruction::Dec16Bits(
                    WordTarget::Registers(RR::HL),
                    Data::new(1, 8, None, opcode),
                ),
            ),

            // INC L
            0x2C => Some(
                Instruction::Inc(
                    ByteTarget::Register(R::L),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // DEC L
            0x2D => Some(
                Instruction::Dec(
                    ByteTarget::Register(R::L),
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // LD L, u8
            0x2E => Some(
                Instruction::Ld(
                    LoadType::Byte(
                        ByteTarget::Register(R::L),
                        ByteSource::Immediate8,
                    ),
                    Data::new(2, 8, None, opcode),
                ),
            ),

            // CPL
            0x2F => Some(
                Instruction::Cpl(
                    Data::new(1, 4, None, opcode),
                ),
            ),

            // ADD A, B
            0x80 => Some(
                Instruction::Add(
                    R::B,
                    Data::new(1, 4, None, opcode),
                )
            ),
            
            // ADD A, C
            0x81 => Some(
                Instruction::Add(
                    R::C,
                    Data::new(1, 4, None, opcode),
                )
            ),
            
            // ADD A, D
            0x82 => Some(
                Instruction::Add(
                    R::D,
                    Data::new(1, 4, None, opcode),
                )
            ),
            
            // ADD A, E
            0x83 => Some(
                Instruction::Add(
                    R::E,
                    Data::new(1, 4, None, opcode),
                )
            ),
            
            // ADD A, H
            0x84 => Some(
                Instruction::Add(
                    R::H,
                    Data::new(1, 4, None, opcode),
                )
            ),
            
            // ADD A, L
            0x85 => Some(
                Instruction::Add(
                    R::L,
                    Data::new(1, 4, None, opcode),
                )
            ),
            
            // JP nz, u16
            0xC2 => Some(
                Instruction::Jp(
                    JumpTest::NotZero,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), opcode),
                )
            ),
            
            // JP n16
            0xC3 => Some(
                Instruction::Jp(
                    JumpTest::Always,
                    WordSource::Immediate16,
                    Data::new(3, 16, Some(16), opcode),
                )
            ),
            
            // JP z, n16
            0xCA => Some(
                Instruction::Jp(
                    JumpTest::Zero,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), opcode),
                )
            ),

            // JP nc, n16
            0xD2 => Some(
                Instruction::Jp(
                    JumpTest::NotCarry,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), opcode),
                )
            ),

            // JP cc, n16
            0xDA => Some(
                Instruction::Jp(
                    JumpTest::Carry,
                    WordSource::Immediate16,
                    Data::new(3, 12, Some(16), opcode),
                )
            ),

            // JP HL
            0xE9 => Some(
                Instruction::Jp(
                    JumpTest::Always,
                    WordSource::Registers(RR::HL),
                    Data::new(1, 4, Some(4), opcode),
                )
            ),

            // Unknown opcode
            _ => None,
        }
    }
}
