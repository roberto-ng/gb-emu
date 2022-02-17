#[derive(Copy, Clone, Debug)]
pub struct Data {
    pub bytes: u16,
    pub cycles: u32,
    // The duration of conditional calls and returns is depending on if the action is taken or not
    pub action_cycles: Option<u32>,
    pub opcode: u8,
    pub is_prefixed: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum Instruction {
    AdC(ByteSource, Data),
    Add(ByteSource, Data),
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
    Direct,
    HL,
    HLI,
    HLD,
    FF00PlusC,
    FF00PlusU8,
}

#[derive(Copy, Clone, Debug)]
pub enum ByteSource {
    Register(R),
    Registers(RR),
    Immediate8,
    Direct,
    HL,
    HLI,
    HLD,
    FF00PlusC,
    FF00PlusU8,
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
    pub const fn new(bytes: u16, cycles: u32, action_cycles: Option<u32>, opcode: u8) -> Data {
        Data {
            bytes,
            cycles,
            action_cycles,
            opcode,
            is_prefixed: false,
        }
    }

    // new 0xCB prefixed instruction
    pub const fn new_cb(bytes: u16, cycles: u32, action_cycles: Option<u32>, opcode: u8) -> Data {
        Data {
            bytes,
            cycles,
            action_cycles,
            opcode,
            is_prefixed: true,
        }
    }

    pub fn get_action_cycles(&self) -> u32 {
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
            }
        }
    }
}

impl Instruction {
    pub const fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        if prefixed {
            Instruction::from_byte_0xcb_prefixed(byte)
        } else {
            Instruction::from_byte_not_prefixed(byte)
        }
    }

    const fn from_byte_not_prefixed(opcode: u8) -> Option<Instruction> {
        match opcode {
            // NOP
            0x00 => Some(Instruction::NoOp(Data::new(1, 4, None, opcode))),

            // LD BC, u16
            0x01 => Some(Instruction::Ld(
                LoadType::Word(WordTarget::Registers(RR::BC), WordSource::Immediate16),
                Data::new(3, 12, None, opcode),
            )),

            // LD (BC), A
            0x02 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Registers(RR::BC), ByteSource::Register(R::A)),
                Data::new(1, 8, None, opcode),
            )),

            // INC BC
            0x03 => Some(Instruction::Inc(
                ByteTarget::Registers(RR::BC),
                Data::new(1, 8, None, opcode),
            )),

            // INC B
            0x04 => Some(Instruction::Inc(
                ByteTarget::Register(R::B),
                Data::new(1, 4, None, opcode),
            )),

            // DEC B
            0x05 => Some(Instruction::Dec(
                ByteTarget::Register(R::B),
                Data::new(1, 4, None, opcode),
            )),

            // LD B, u8
            0x06 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::B), ByteSource::Immediate8),
                Data::new(2, 8, None, opcode),
            )),

            // RLCA
            0x07 => Some(Instruction::RLC(
                ByteTarget::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // LD (u16), SP
            0x08 => Some(Instruction::Ld(
                LoadType::Word(WordTarget::Direct, WordSource::SP),
                Data::new(3, 20, None, opcode),
            )),

            // ADD HL, BC
            0x09 => Some(Instruction::Add16Bits(
                WordSource::Registers(RR::BC),
                WordTarget::Registers(RR::HL),
                Data::new(1, 8, None, opcode),
            )),

            // LD A, (BC)
            0x0A => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Registers(RR::BC)),
                Data::new(1, 8, None, opcode),
            )),

            // DEC BC
            0x0B => Some(Instruction::Dec16Bits(
                WordTarget::Registers(RR::BC),
                Data::new(1, 8, None, opcode),
            )),

            // INC C
            0x0C => Some(Instruction::Inc(
                ByteTarget::Register(R::C),
                Data::new(1, 4, None, opcode),
            )),

            // DEC C
            0x0D => Some(Instruction::Dec(
                ByteTarget::Register(R::C),
                Data::new(1, 4, None, opcode),
            )),

            // LD C, u8
            0x0E => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::C), ByteSource::Immediate8),
                Data::new(2, 8, None, opcode),
            )),

            // RRCA
            0x0F => Some(Instruction::RRC(
                ByteTarget::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // STOP
            0x10 => Some(Instruction::Stop(Data::new(1, 4, None, opcode))),

            // LD DE, u16
            0x11 => Some(Instruction::Ld(
                LoadType::Word(WordTarget::Registers(RR::DE), WordSource::Immediate16),
                Data::new(3, 12, None, opcode),
            )),

            // LD (DE),A
            0x12 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Registers(RR::DE), ByteSource::Register(R::A)),
                Data::new(1, 8, None, opcode),
            )),

            // INC DE
            0x13 => Some(Instruction::Inc16Bits(
                WordTarget::Registers(RR::DE),
                Data::new(1, 8, None, opcode),
            )),

            // INC D
            0x14 => Some(Instruction::Inc(
                ByteTarget::Register(R::D),
                Data::new(1, 4, None, opcode),
            )),

            // DEC D
            0x15 => Some(Instruction::Dec(
                ByteTarget::Register(R::D),
                Data::new(1, 4, None, opcode),
            )),

            // LD D, u8
            0x16 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::D), ByteSource::Immediate8),
                Data::new(2, 8, None, opcode),
            )),

            // RLA
            0x17 => Some(Instruction::RL(
                ByteTarget::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // JR i8
            0x18 => Some(Instruction::JR(
                JumpTest::Always,
                Data::new(2, 12, Some(12), opcode),
            )),

            // ADD HL, DE
            0x19 => Some(Instruction::Add16Bits(
                WordSource::Registers(RR::DE),
                WordTarget::Registers(RR::HL),
                Data::new(1, 8, None, opcode),
            )),

            // LD A, (DE)
            0x1A => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Registers(RR::DE)),
                Data::new(1, 8, None, opcode),
            )),

            // DEC DE
            0x1B => Some(Instruction::Dec16Bits(
                WordTarget::Registers(RR::DE),
                Data::new(1, 8, None, opcode),
            )),

            // INC E
            0x1C => Some(Instruction::Inc(
                ByteTarget::Register(R::E),
                Data::new(1, 4, None, opcode),
            )),

            // DEC E
            0x1D => Some(Instruction::Dec(
                ByteTarget::Register(R::E),
                Data::new(1, 4, None, opcode),
            )),

            // LD E, u8
            0x1E => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::E), ByteSource::Immediate8),
                Data::new(2, 8, None, opcode),
            )),

            // RRA
            0x1F => Some(Instruction::RR(
                ByteTarget::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // JR NZ, i8
            0x20 => Some(Instruction::JR(
                JumpTest::NotZero,
                Data::new(2, 8, Some(12), opcode),
            )),

            // LD HL, u16
            0x21 => Some(Instruction::Ld(
                LoadType::Word(WordTarget::Registers(RR::HL), WordSource::Immediate16),
                Data::new(3, 12, None, opcode),
            )),

            // LD (HL+),A
            0x22 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::HLI, ByteSource::Register(R::A)),
                Data::new(1, 8, None, opcode),
            )),

            // INC HL
            0x23 => Some(Instruction::Inc16Bits(
                WordTarget::Registers(RR::HL),
                Data::new(1, 8, None, opcode),
            )),

            // INC H
            0x24 => Some(Instruction::Inc(
                ByteTarget::Register(R::H),
                Data::new(1, 4, None, opcode),
            )),

            // DEC H
            0x25 => Some(Instruction::Dec(
                ByteTarget::Register(R::H),
                Data::new(1, 4, None, opcode),
            )),

            // LD H, u8
            0x26 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::H), ByteSource::Immediate8),
                Data::new(2, 8, None, opcode),
            )),

            // DAA
            0x27 => Some(Instruction::DAA(Data::new(1, 4, None, opcode))),

            // JR Z, i8
            0x28 => Some(Instruction::JR(
                JumpTest::Zero,
                Data::new(2, 8, Some(12), opcode),
            )),

            // ADD HL, HL
            0x29 => Some(Instruction::Add16Bits(
                WordSource::Registers(RR::HL),
                WordTarget::Registers(RR::HL),
                Data::new(1, 8, None, opcode),
            )),

            // LD A, (HL+)
            0x2A => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::HLI),
                Data::new(1, 8, None, opcode),
            )),

            // DEC HL
            0x2B => Some(Instruction::Dec16Bits(
                WordTarget::Registers(RR::HL),
                Data::new(1, 8, None, opcode),
            )),

            // INC L
            0x2C => Some(Instruction::Inc(
                ByteTarget::Register(R::L),
                Data::new(1, 4, None, opcode),
            )),

            // DEC L
            0x2D => Some(Instruction::Dec(
                ByteTarget::Register(R::L),
                Data::new(1, 4, None, opcode),
            )),

            // LD L, u8
            0x2E => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::L), ByteSource::Immediate8),
                Data::new(2, 8, None, opcode),
            )),

            // CPL
            0x2F => Some(Instruction::Cpl(Data::new(1, 4, None, opcode))),

            // JR NC, i8
            0x30 => Some(Instruction::JR(
                JumpTest::NotCarry,
                Data::new(2, 8, Some(12), opcode),
            )),

            // LD SP, u16
            0x31 => Some(Instruction::Ld(
                LoadType::Word(WordTarget::SP, WordSource::Immediate16),
                Data::new(3, 12, None, opcode),
            )),

            // LD (HL-), A
            0x32 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::HLD, ByteSource::Register(R::A)),
                Data::new(1, 8, None, opcode),
            )),

            // INC SP
            0x33 => Some(Instruction::Inc16Bits(
                WordTarget::SP,
                Data::new(1, 8, None, opcode),
            )),

            // INC (HL)
            0x34 => Some(Instruction::Inc(
                ByteTarget::HL,
                Data::new(1, 12, None, opcode),
            )),

            // DEC (HL)
            0x35 => Some(Instruction::Dec(
                ByteTarget::HL,
                Data::new(1, 12, None, opcode),
            )),

            // LD (HL), u8
            0x36 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::HL, ByteSource::Immediate8),
                Data::new(2, 12, None, opcode),
            )),

            // SCF
            0x37 => Some(Instruction::SCF(Data::new(1, 4, None, opcode))),

            // JR C, i8
            0x38 => Some(Instruction::JR(
                JumpTest::Carry,
                Data::new(2, 8, Some(12), opcode),
            )),

            // ADD HL, SP
            0x39 => Some(Instruction::Add16Bits(
                WordSource::SP,
                WordTarget::Registers(RR::HL),
                Data::new(1, 8, None, opcode),
            )),

            // LD A, (HL-)
            0x3A => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::HLD),
                Data::new(1, 8, None, opcode),
            )),

            // DEC SP
            0x3B => Some(Instruction::Dec16Bits(
                WordTarget::SP,
                Data::new(1, 8, None, opcode),
            )),

            // INC A
            0x3C => Some(Instruction::Inc(
                ByteTarget::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // DEC A
            0x3D => Some(Instruction::Dec(
                ByteTarget::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // LD A, u8
            0x3E => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Immediate8),
                Data::new(2, 2, None, opcode),
            )),

            // CCF
            0x3F => Some(Instruction::CCF(Data::new(1, 4, None, opcode))),

            // LD B,B
            0x40 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::B), ByteSource::Register(R::B)),
                Data::new(1, 4, None, opcode),
            )),

            // LD B,C
            0x41 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::B), ByteSource::Register(R::C)),
                Data::new(1, 4, None, opcode),
            )),

            // LD B, D
            0x42 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::B), ByteSource::Register(R::D)),
                Data::new(1, 4, None, opcode),
            )),

            // LD B, E
            0x43 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::B), ByteSource::Register(R::E)),
                Data::new(1, 4, None, opcode),
            )),

            // LD B, H
            0x44 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::B), ByteSource::Register(R::H)),
                Data::new(1, 4, None, opcode),
            )),

            // LD B, L
            0x45 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::B), ByteSource::Register(R::L)),
                Data::new(1, 4, None, opcode),
            )),

            // LD B, (HL)
            0x46 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::B), ByteSource::HL),
                Data::new(1, 8, None, opcode),
            )),

            // LD B, A
            0x47 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::B), ByteSource::Register(R::A)),
                Data::new(1, 4, None, opcode),
            )),

            // LD C, B
            0x48 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::C), ByteSource::Register(R::B)),
                Data::new(1, 4, None, opcode),
            )),

            // LD C,C
            0x49 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::C), ByteSource::Register(R::C)),
                Data::new(1, 4, None, opcode),
            )),

            // LD A, (HL-)
            0x4A => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::HLD),
                Data::new(1, 8, None, opcode),
            )),

            // LD C,E
            0x4B => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::C), ByteSource::Register(R::E)),
                Data::new(1, 4, None, opcode),
            )),

            // LD C, H
            0x4C => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::C), ByteSource::Register(R::H)),
                Data::new(1, 4, None, opcode),
            )),

            // LD C, L
            0x4D => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::C), ByteSource::Register(R::L)),
                Data::new(1, 4, None, opcode),
            )),

            // LD C, (HL)
            0x4E => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::C), ByteSource::HL),
                Data::new(1, 4, None, opcode),
            )),

            // LD C, A
            0x4F => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::C), ByteSource::Register(R::A)),
                Data::new(1, 4, None, opcode),
            )),

            // LD D,B
            0x50 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::D), ByteSource::Register(R::B)),
                Data::new(1, 4, None, opcode),
            )),

            // LD D,C
            0x51 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::D), ByteSource::Register(R::C)),
                Data::new(1, 4, None, opcode),
            )),

            // LD D,D
            0x52 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::D), ByteSource::Register(R::D)),
                Data::new(1, 4, None, opcode),
            )),

            // LD D, E
            0x53 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::D), ByteSource::Register(R::E)),
                Data::new(1, 4, None, opcode),
            )),

            // LD D, H
            0x54 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::D), ByteSource::Register(R::H)),
                Data::new(1, 4, None, opcode),
            )),

            // LD D,L
            0x55 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::D), ByteSource::Register(R::L)),
                Data::new(1, 4, None, opcode),
            )),

            // LD D, (HL)
            0x56 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::D), ByteSource::HL),
                Data::new(1, 8, None, opcode),
            )),

            // LD D, A
            0x57 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::D), ByteSource::Register(R::A)),
                Data::new(1, 4, None, opcode),
            )),

            // LD E, B
            0x58 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::E), ByteSource::Register(R::B)),
                Data::new(1, 4, None, opcode),
            )),

            // LD E, C
            0x59 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::E), ByteSource::Register(R::C)),
                Data::new(1, 4, None, opcode),
            )),

            // LD E, D
            0x5A => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::E), ByteSource::Register(R::D)),
                Data::new(1, 4, None, opcode),
            )),

            // LD E,E
            0x5B => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::E), ByteSource::Register(R::E)),
                Data::new(1, 4, None, opcode),
            )),

            // LD E, H
            0x5C => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::E), ByteSource::Register(R::H)),
                Data::new(1, 4, None, opcode),
            )),

            // LD E,L
            0x5D => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::E), ByteSource::Register(R::L)),
                Data::new(1, 4, None, opcode),
            )),

            // LD E,(HL)
            0x5E => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::E), ByteSource::HL),
                Data::new(1, 8, None, opcode),
            )),

            // LD E, A
            0x5F => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::E), ByteSource::Register(R::A)),
                Data::new(1, 4, None, opcode),
            )),

            // LD H, B
            0x60 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::H), ByteSource::Register(R::B)),
                Data::new(1, 4, None, opcode),
            )),

            // LD H, C
            0x61 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::H), ByteSource::Register(R::C)),
                Data::new(1, 4, None, opcode),
            )),

            // LD H, D
            0x62 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::H), ByteSource::Register(R::D)),
                Data::new(1, 4, None, opcode),
            )),

            // LD H, E
            0x63 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::H), ByteSource::Register(R::E)),
                Data::new(1, 4, None, opcode),
            )),

            // LD H, H
            0x64 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::H), ByteSource::Register(R::H)),
                Data::new(1, 4, None, opcode),
            )),

            // LD H, L
            0x65 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::H), ByteSource::Register(R::L)),
                Data::new(1, 4, None, opcode),
            )),

            // LD H, (HL)
            0x66 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::H), ByteSource::HL),
                Data::new(1, 8, None, opcode),
            )),

            // LD H, A
            0x67 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::H), ByteSource::Register(R::A)),
                Data::new(1, 4, None, opcode),
            )),

            // LD L, B
            0x68 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::L), ByteSource::Register(R::B)),
                Data::new(1, 4, None, opcode),
            )),

            // LD L, C
            0x69 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::L), ByteSource::Register(R::C)),
                Data::new(1, 4, None, opcode),
            )),

            // LD L, D
            0x6A => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::L), ByteSource::Register(R::D)),
                Data::new(1, 4, None, opcode),
            )),

            // LD L, E
            0x6B => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::L), ByteSource::Register(R::E)),
                Data::new(1, 4, None, opcode),
            )),

            // LD L, H
            0x6C => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::L), ByteSource::Register(R::H)),
                Data::new(1, 4, None, opcode),
            )),

            // LD L, L
            0x6D => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::L), ByteSource::Register(R::L)),
                Data::new(1, 4, None, opcode),
            )),

            // LD L, (HL)
            0x6E => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::L), ByteSource::HL),
                Data::new(1, 8, None, opcode),
            )),

            // LD L, A
            0x6F => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::L), ByteSource::Register(R::A)),
                Data::new(1, 4, None, opcode),
            )),

            // LD (HL), B
            0x70 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::HL, ByteSource::Register(R::B)),
                Data::new(1, 8, None, opcode),
            )),

            // LD (HL), C
            0x71 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::HL, ByteSource::Register(R::C)),
                Data::new(1, 8, None, opcode),
            )),

            // LD (HL), D
            0x72 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::HL, ByteSource::Register(R::D)),
                Data::new(1, 8, None, opcode),
            )),

            //LD (HL), E
            0x73 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::HL, ByteSource::Register(R::E)),
                Data::new(1, 8, None, opcode),
            )),

            // LD (HL), H
            0x74 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::HL, ByteSource::Register(R::H)),
                Data::new(1, 8, None, opcode),
            )),

            // LD (HL), L
            0x75 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::HL, ByteSource::Register(R::L)),
                Data::new(1, 8, None, opcode),
            )),

            // HALT
            0x76 => Some(Instruction::Halt(Data::new(1, 4, None, opcode))),

            // LD (HL), A
            0x77 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::HL, ByteSource::Register(R::A)),
                Data::new(1, 8, None, opcode),
            )),

            // LD A, B
            0x78 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Register(R::B)),
                Data::new(1, 4, None, opcode),
            )),

            // LD A, C
            0x79 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Register(R::C)),
                Data::new(1, 4, None, opcode),
            )),

            // LD A, D
            0x7A => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Register(R::D)),
                Data::new(1, 4, None, opcode),
            )),

            // LD A, E
            0x7B => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Register(R::E)),
                Data::new(1, 4, None, opcode),
            )),

            // LD A, H
            0x7C => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Register(R::H)),
                Data::new(1, 4, None, opcode),
            )),

            // LD A, L
            0x7D => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Register(R::L)),
                Data::new(1, 4, None, opcode),
            )),

            // LD A, (HL)
            0x7E => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::HL),
                Data::new(1, 4, None, opcode),
            )),

            // LD A, A
            0x7F => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Register(R::A)),
                Data::new(1, 4, None, opcode),
            )),

            // ADD A, B
            0x80 => Some(Instruction::Add(
                ByteSource::Register(R::B),
                Data::new(1, 4, None, opcode),
            )),

            // ADD A, C
            0x81 => Some(Instruction::Add(
                ByteSource::Register(R::C),
                Data::new(1, 4, None, opcode),
            )),

            // ADD A, D
            0x82 => Some(Instruction::Add(
                ByteSource::Register(R::D),
                Data::new(1, 4, None, opcode),
            )),

            // ADD A, E
            0x83 => Some(Instruction::Add(
                ByteSource::Register(R::E),
                Data::new(1, 4, None, opcode),
            )),

            // ADD A, H
            0x84 => Some(Instruction::Add(
                ByteSource::Register(R::H),
                Data::new(1, 4, None, opcode),
            )),

            // ADD A, L
            0x85 => Some(Instruction::Add(
                ByteSource::Register(R::L),
                Data::new(1, 4, None, opcode),
            )),

            // ADD A, (HL)
            0x86 => Some(Instruction::Add(
                ByteSource::HL,
                Data::new(1, 8, None, opcode),
            )),

            // ADD A, A
            0x87 => Some(Instruction::Add(
                ByteSource::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // ADC A, B
            0x88 => Some(Instruction::AdC(
                ByteSource::Register(R::B),
                Data::new(1, 4, None, opcode),
            )),

            // ADC A, C
            0x89 => Some(Instruction::AdC(
                ByteSource::Register(R::C),
                Data::new(1, 4, None, opcode),
            )),

            // ADC A, D
            0x8A => Some(Instruction::AdC(
                ByteSource::Register(R::D),
                Data::new(1, 4, None, opcode),
            )),

            // ADC A, E
            0x8B => Some(Instruction::AdC(
                ByteSource::Register(R::E),
                Data::new(1, 4, None, opcode),
            )),

            // ADC A, H
            0x8C => Some(Instruction::AdC(
                ByteSource::Register(R::H),
                Data::new(1, 4, None, opcode),
            )),

            // ADC A, L
            0x8D => Some(Instruction::AdC(
                ByteSource::Register(R::L),
                Data::new(1, 4, None, opcode),
            )),

            // ADC A, (HL)
            0x8E => Some(Instruction::AdC(
                ByteSource::HL,
                Data::new(1, 8, None, opcode),
            )),

            // ADC A, A
            0x8F => Some(Instruction::AdC(
                ByteSource::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // SUB A, B
            0x90 => Some(Instruction::Sub(
                ByteSource::Register(R::B),
                Data::new(1, 4, None, opcode),
            )),

            // SUB A, C
            0x91 => Some(Instruction::Sub(
                ByteSource::Register(R::C),
                Data::new(1, 4, None, opcode),
            )),

            // SUB A, D
            0x92 => Some(Instruction::Sub(
                ByteSource::Register(R::D),
                Data::new(1, 4, None, opcode),
            )),

            // SUB A, E
            0x93 => Some(Instruction::Sub(
                ByteSource::Register(R::E),
                Data::new(1, 4, None, opcode),
            )),

            // SUB A, H
            0x94 => Some(Instruction::Sub(
                ByteSource::Register(R::H),
                Data::new(1, 4, None, opcode),
            )),

            // SUB A, L
            0x95 => Some(Instruction::Sub(
                ByteSource::Register(R::L),
                Data::new(1, 4, None, opcode),
            )),

            // SUB A, (HL)
            0x96 => Some(Instruction::Sub(
                ByteSource::HL,
                Data::new(1, 8, None, opcode),
            )),

            // SUB A, A
            0x97 => Some(Instruction::Sub(
                ByteSource::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // SBC A, B
            0x98 => Some(Instruction::SbC(
                ByteSource::Register(R::B),
                Data::new(1, 4, None, opcode),
            )),

            // SBC A, C
            0x99 => Some(Instruction::SbC(
                ByteSource::Register(R::C),
                Data::new(1, 4, None, opcode),
            )),

            // SBC A, D
            0x9A => Some(Instruction::SbC(
                ByteSource::Register(R::D),
                Data::new(1, 4, None, opcode),
            )),

            // SBC A, E
            0x9B => Some(Instruction::SbC(
                ByteSource::Register(R::E),
                Data::new(1, 4, None, opcode),
            )),

            // SBC A, H
            0x9C => Some(Instruction::SbC(
                ByteSource::Register(R::H),
                Data::new(1, 4, None, opcode),
            )),

            // SBC A, L
            0x9D => Some(Instruction::SbC(
                ByteSource::Register(R::L),
                Data::new(1, 4, None, opcode),
            )),

            // SBC A, (HL)
            0x9E => Some(Instruction::SbC(
                ByteSource::HL,
                Data::new(1, 8, None, opcode),
            )),

            // SBC A, A
            0x9F => Some(Instruction::SbC(
                ByteSource::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // AND A, B
            0xA0 => Some(Instruction::And(
                ByteSource::Register(R::B),
                Data::new(1, 4, None, opcode),
            )),

            // AND A, C
            0xA1 => Some(Instruction::And(
                ByteSource::Register(R::C),
                Data::new(1, 4, None, opcode),
            )),

            // AND A, D
            0xA2 => Some(Instruction::And(
                ByteSource::Register(R::D),
                Data::new(1, 4, None, opcode),
            )),

            // AND A, E
            0xA3 => Some(Instruction::And(
                ByteSource::Register(R::E),
                Data::new(1, 4, None, opcode),
            )),

            // AND A, H
            0xA4 => Some(Instruction::And(
                ByteSource::Register(R::H),
                Data::new(1, 4, None, opcode),
            )),

            // AND A, L
            0xA5 => Some(Instruction::And(
                ByteSource::Register(R::L),
                Data::new(1, 4, None, opcode),
            )),

            // AND A, (HL)
            0xA6 => Some(Instruction::And(
                ByteSource::HL,
                Data::new(1, 8, None, opcode),
            )),

            // AND A, A
            0xA7 => Some(Instruction::And(
                ByteSource::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // XOR A, B
            0xA8 => Some(Instruction::XOr(
                ByteSource::Register(R::B),
                Data::new(1, 4, None, opcode),
            )),

            // XOR A, C
            0xA9 => Some(Instruction::XOr(
                ByteSource::Register(R::C),
                Data::new(1, 4, None, opcode),
            )),

            // XOR A, D
            0xAA => Some(Instruction::XOr(
                ByteSource::Register(R::D),
                Data::new(1, 4, None, opcode),
            )),

            // XOR A, E
            0xAB => Some(Instruction::XOr(
                ByteSource::Register(R::E),
                Data::new(1, 4, None, opcode),
            )),

            // XOR A, H
            0xAC => Some(Instruction::XOr(
                ByteSource::Register(R::H),
                Data::new(1, 4, None, opcode),
            )),

            // XOR A, L
            0xAD => Some(Instruction::XOr(
                ByteSource::Register(R::L),
                Data::new(1, 4, None, opcode),
            )),

            // XOR A, (HL)
            0xAE => Some(Instruction::XOr(
                ByteSource::HL,
                Data::new(1, 8, None, opcode),
            )),

            // XOR A, A
            0xAF => Some(Instruction::XOr(
                ByteSource::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // OR A, B
            0xB0 => Some(Instruction::Or(
                ByteSource::Register(R::B),
                Data::new(1, 4, None, opcode),
            )),

            // OR A, C
            0xB1 => Some(Instruction::Or(
                ByteSource::Register(R::C),
                Data::new(1, 4, None, opcode),
            )),

            // OR A, D
            0xB2 => Some(Instruction::Or(
                ByteSource::Register(R::D),
                Data::new(1, 4, None, opcode),
            )),

            // OR A, E
            0xB3 => Some(Instruction::Or(
                ByteSource::Register(R::E),
                Data::new(1, 4, None, opcode),
            )),

            // OR A, H
            0xB4 => Some(Instruction::Or(
                ByteSource::Register(R::H),
                Data::new(1, 4, None, opcode),
            )),

            // OR A, L
            0xB5 => Some(Instruction::Or(
                ByteSource::Register(R::L),
                Data::new(1, 4, None, opcode),
            )),

            // OR A, (HL)
            0xB6 => Some(Instruction::Or(
                ByteSource::HL,
                Data::new(1, 8, None, opcode),
            )),

            // OR A, A
            0xB7 => Some(Instruction::Or(
                ByteSource::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // CP A, B
            0xB8 => Some(Instruction::Cp(
                ByteSource::Register(R::B),
                Data::new(1, 4, None, opcode),
            )),

            // CP A, C
            0xB9 => Some(Instruction::Cp(
                ByteSource::Register(R::C),
                Data::new(1, 4, None, opcode),
            )),

            // CP A, D
            0xBA => Some(Instruction::Cp(
                ByteSource::Register(R::D),
                Data::new(1, 4, None, opcode),
            )),

            // CP A, E
            0xBB => Some(Instruction::Cp(
                ByteSource::Register(R::E),
                Data::new(1, 4, None, opcode),
            )),

            // CP A, H
            0xBC => Some(Instruction::Cp(
                ByteSource::Register(R::H),
                Data::new(1, 4, None, opcode),
            )),

            // CP A, L
            0xBD => Some(Instruction::Cp(
                ByteSource::Register(R::L),
                Data::new(1, 4, None, opcode),
            )),

            // CP A, (HL)
            0xBE => Some(Instruction::Cp(
                ByteSource::HL,
                Data::new(1, 8, None, opcode),
            )),

            // CP A, A
            0xBF => Some(Instruction::Cp(
                ByteSource::Register(R::A),
                Data::new(1, 4, None, opcode),
            )),

            // RET NZ
            0xC0 => Some(Instruction::Ret(
                JumpTest::NotZero,
                Data::new(1, 8, Some(20), opcode),
            )),

            // POP BC
            0xC1 => Some(Instruction::Pop(RR::BC, Data::new(1, 12, None, opcode))),

            // JP nz, u16
            0xC2 => Some(Instruction::Jp(
                JumpTest::NotZero,
                WordSource::Immediate16,
                Data::new(3, 12, Some(16), opcode),
            )),

            // JP n16
            0xC3 => Some(Instruction::Jp(
                JumpTest::Always,
                WordSource::Immediate16,
                Data::new(3, 16, Some(16), opcode),
            )),

            // CALL NZ, u16
            0xC4 => Some(Instruction::Call(
                JumpTest::NotZero,
                Data::new(3, 12, Some(24), opcode),
            )),

            // PUSH BC
            0xC5 => Some(Instruction::Push(RR::BC, Data::new(1, 16, None, opcode))),

            // ADD A, u8
            0xC6 => Some(Instruction::Add(
                ByteSource::Immediate8,
                Data::new(2, 8, None, opcode),
            )),

            // RST 00h
            0xC7 => Some(Instruction::RST(0x00, Data::new(1, 16, None, opcode))),

            // RET Z
            0xC8 => Some(Instruction::Ret(
                JumpTest::Zero,
                Data::new(1, 8, Some(20), opcode),
            )),

            // RET
            0xC9 => Some(Instruction::Ret(
                JumpTest::Always,
                Data::new(1, 16, Some(16), opcode),
            )),

            // JP z, n16
            0xCA => Some(Instruction::Jp(
                JumpTest::Zero,
                WordSource::Immediate16,
                Data::new(3, 12, Some(16), opcode),
            )),

            // 0xCB is used as a prefix for other instructions, so let's skip it.

            // CALL Z, u16
            0xCC => Some(Instruction::Call(
                JumpTest::Zero,
                Data::new(3, 12, Some(24), opcode),
            )),

            // CALL u16
            0xCD => Some(Instruction::Call(
                JumpTest::Always,
                Data::new(3, 24, Some(24), opcode),
            )),

            // ADC A, u8
            0xCE => Some(Instruction::AdC(
                ByteSource::Immediate8,
                Data::new(2, 8, None, opcode),
            )),

            // RST 08h
            0xCF => Some(Instruction::RST(0x08, Data::new(1, 16, None, opcode))),

            // RET NC
            0xD0 => Some(Instruction::Ret(
                JumpTest::NotCarry,
                Data::new(1, 8, Some(20), opcode),
            )),

            // POP DE
            0xD1 => Some(Instruction::Pop(RR::DE, Data::new(1, 12, None, opcode))),

            // JP nc, n16
            0xD2 => Some(Instruction::Jp(
                JumpTest::NotCarry,
                WordSource::Immediate16,
                Data::new(3, 12, Some(16), opcode),
            )),

            // CALL NC, u16
            0xD4 => Some(Instruction::Call(
                JumpTest::NotCarry,
                Data::new(3, 12, Some(24), opcode),
            )),

            // PUSH DE
            0xD5 => Some(Instruction::Push(RR::DE, Data::new(1, 16, None, opcode))),

            // SUB A, u8
            0xD6 => Some(Instruction::Sub(
                ByteSource::Immediate8,
                Data::new(2, 8, None, opcode),
            )),

            // RST 10h
            0xD7 => Some(Instruction::RST(0x10, Data::new(1, 16, None, opcode))),

            // RET C
            0xD8 => Some(Instruction::Ret(
                JumpTest::Carry,
                Data::new(1, 8, Some(20), opcode),
            )),

            // RETI
            0xD9 => Some(Instruction::RetI(Data::new(1, 16, None, opcode))),

            // JP C, u16
            0xDA => Some(Instruction::Jp(
                JumpTest::Carry,
                WordSource::Immediate16,
                Data::new(3, 12, Some(16), opcode),
            )),

            // CALL C, u16
            0xDC => Some(Instruction::Call(
                JumpTest::Carry,
                Data::new(3, 12, Some(24), opcode),
            )),

            // SBC A, u8
            0xDE => Some(Instruction::SbC(
                ByteSource::Immediate8,
                Data::new(2, 8, None, opcode),
            )),

            // RST 18h
            0xDF => Some(Instruction::RST(0x18, Data::new(1, 16, None, opcode))),

            // LD (FF00+u8), A
            0xE0 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::FF00PlusU8, ByteSource::Register(R::A)),
                Data::new(2, 12, None, opcode),
            )),

            // POP HL
            0xE1 => Some(Instruction::Pop(RR::HL, Data::new(1, 12, None, opcode))),

            // LD (FF00+C), A
            0xE2 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::FF00PlusC, ByteSource::Register(R::A)),
                Data::new(1, 8, None, opcode),
            )),

            // PUSH HL
            0xE5 => Some(Instruction::Push(RR::HL, Data::new(1, 16, None, opcode))),

            // AND A, u8
            0xE6 => Some(Instruction::And(
                ByteSource::Immediate8,
                Data::new(2, 8, None, opcode),
            )),

            // RST 20h
            0xE7 => Some(Instruction::RST(0x20, Data::new(1, 16, None, opcode))),

            // ADD SP, i8
            0xE8 => Some(Instruction::Add(
                ByteSource::Immediate8,
                Data::new(2, 16, None, opcode),
            )),

            // JP HL
            0xE9 => Some(Instruction::Jp(
                JumpTest::Always,
                WordSource::Registers(RR::HL),
                Data::new(1, 4, Some(4), opcode),
            )),

            // LD (u16), A
            0xEA => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Direct, ByteSource::Register(R::A)),
                Data::new(3, 16, None, opcode),
            )),

            // XOR A, u8
            0xEE => Some(Instruction::XOr(
                ByteSource::Immediate8,
                Data::new(2, 8, None, opcode),
            )),

            // RST 28h
            0xEF => Some(Instruction::RST(0x28, Data::new(1, 16, None, opcode))),

            // LD A, (FF00+u8)
            0xF0 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::FF00PlusU8),
                Data::new(2, 12, None, opcode),
            )),

            // POP AF
            0xF1 => Some(Instruction::Pop(RR::AF, Data::new(1, 12, None, opcode))),

            // LD A, (FF00+C)
            0xF2 => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::FF00PlusC),
                Data::new(1, 8, None, opcode),
            )),

            // DI
            0xF3 => Some(Instruction::DI(Data::new(1, 4, None, opcode))),

            // PUSH AF
            0xF5 => Some(Instruction::Push(RR::AF, Data::new(1, 16, None, opcode))),

            // OR A, u8
            0xF6 => Some(Instruction::Or(
                ByteSource::Immediate8,
                Data::new(2, 8, None, opcode),
            )),

            // RST 30h
            0xF7 => Some(Instruction::RST(0x30, Data::new(1, 16, None, opcode))),

            // LD HL, SP+i8
            0xF8 => Some(Instruction::Ld(
                LoadType::Word(WordTarget::Registers(RR::HL), WordSource::SpPlusI8),
                Data::new(2, 12, None, opcode),
            )),

            // LD SP, HL
            0xF9 => Some(Instruction::Ld(
                LoadType::Word(WordTarget::SP, WordSource::Registers(RR::HL)),
                Data::new(1, 8, None, opcode),
            )),

            // LD A, (u16)
            0xFA => Some(Instruction::Ld(
                LoadType::Byte(ByteTarget::Register(R::A), ByteSource::Direct),
                Data::new(3, 8, None, opcode),
            )),

            // EI
            0xFB => Some(Instruction::EI(Data::new(1, 4, None, opcode))),

            // CP A, u8
            0xFE => Some(Instruction::Cp(
                ByteSource::Immediate8,
                Data::new(2, 8, None, opcode),
            )),

            // RST 38h
            0xFF => Some(Instruction::RST(0x38, Data::new(1, 16, None, opcode))),

            // Unknown opcode
            _ => None,
        }
    }

    const fn from_byte_0xcb_prefixed(opcode: u8) -> Option<Instruction> {
        match opcode {
            // RLC B
            0x00 => Some(Instruction::RLC(
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RLC C
            0x01 => Some(Instruction::RLC(
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RLC D
            0x02 => Some(Instruction::RLC(
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RLC E
            0x03 => Some(Instruction::RLC(
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RLC H
            0x04 => Some(Instruction::RLC(
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RLC L
            0x05 => Some(Instruction::RLC(
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RLC (HL)
            0x06 => Some(Instruction::RLC(
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RLC A
            0x07 => Some(Instruction::RLC(
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RRC B
            0x08 => Some(Instruction::RRC(
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RRC C
            0x09 => Some(Instruction::RRC(
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RRC D
            0x0A => Some(Instruction::RRC(
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RRC E
            0x0B => Some(Instruction::RRC(
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RRC H
            0x0C => Some(Instruction::RRC(
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RRC L
            0x0D => Some(Instruction::RRC(
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RRC (HL)
            0x0E => Some(Instruction::RRC(
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RRC A
            0x0F => Some(Instruction::RRC(
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RL B
            0x10 => Some(Instruction::RL(
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RL C
            0x11 => Some(Instruction::RL(
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RL D
            0x12 => Some(Instruction::RL(
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RL E
            0x13 => Some(Instruction::RL(
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RL H
            0x14 => Some(Instruction::RL(
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RL L
            0x15 => Some(Instruction::RL(
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RL (HL)
            0x16 => Some(Instruction::RL(
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RL A
            0x17 => Some(Instruction::RL(
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RR B
            0x18 => Some(Instruction::RR(
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RR C
            0x19 => Some(Instruction::RR(
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RR D
            0x1A => Some(Instruction::RR(
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RR E
            0x1B => Some(Instruction::RR(
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RR H
            0x1C => Some(Instruction::RR(
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RR L
            0x1D => Some(Instruction::RR(
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RR (HL)
            0x1E => Some(Instruction::RR(
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RR A
            0x1F => Some(Instruction::RR(
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SLA B
            0x20 => Some(Instruction::SLA(
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SLA C
            0x21 => Some(Instruction::SLA(
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SLA D
            0x22 => Some(Instruction::SLA(
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SLA E
            0x23 => Some(Instruction::SLA(
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SLA H
            0x24 => Some(Instruction::SLA(
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SLA L
            0x25 => Some(Instruction::SLA(
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SLA (HL)
            0x26 => Some(Instruction::SLA(
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SLA A
            0x27 => Some(Instruction::SLA(
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRA B
            0x28 => Some(Instruction::SRA(
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRA C
            0x29 => Some(Instruction::SRA(
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRA D
            0x2A => Some(Instruction::SRA(
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRA E
            0x2B => Some(Instruction::SRA(
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRA H
            0x2C => Some(Instruction::SRA(
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRA L
            0x2D => Some(Instruction::SRA(
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRA (HL)
            0x2E => Some(Instruction::SRA(
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SRA A
            0x2F => Some(Instruction::SRA(
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SWAP B
            0x30 => Some(Instruction::Swap(
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SWAP C
            0x31 => Some(Instruction::Swap(
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SWAP D
            0x32 => Some(Instruction::Swap(
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SWAP E
            0x33 => Some(Instruction::Swap(
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SWAP H
            0x34 => Some(Instruction::Swap(
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SWAP L
            0x35 => Some(Instruction::Swap(
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SWAP (HL)
            0x36 => Some(Instruction::Swap(
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SWAP A
            0x37 => Some(Instruction::Swap(
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRL B
            0x38 => Some(Instruction::SRL(
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRL C
            0x39 => Some(Instruction::SRL(
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRL D
            0x3A => Some(Instruction::SRL(
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRL E
            0x3B => Some(Instruction::SRL(
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRL H
            0x3C => Some(Instruction::SRL(
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRL L
            0x3D => Some(Instruction::SRL(
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SRL (HL)
            0x3E => Some(Instruction::SRL(
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SRL A
            0x3F => Some(Instruction::SRL(
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 0, B
            0x40 => Some(Instruction::Bit(
                0,
                ByteSource::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 0, C
            0x41 => Some(Instruction::Bit(
                0,
                ByteSource::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 0, D
            0x42 => Some(Instruction::Bit(
                0,
                ByteSource::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 0, E
            0x43 => Some(Instruction::Bit(
                0,
                ByteSource::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 0, H
            0x44 => Some(Instruction::Bit(
                0,
                ByteSource::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 0, L
            0x45 => Some(Instruction::Bit(
                0,
                ByteSource::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 0, (HL)
            0x46 => Some(Instruction::Bit(
                0,
                ByteSource::HL,
                Data::new_cb(2, 12, None, opcode),
            )),

            // BIT 0, A
            0x47 => Some(Instruction::Bit(
                0,
                ByteSource::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 1, B
            0x48 => Some(Instruction::Bit(
                1,
                ByteSource::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 1, C
            0x49 => Some(Instruction::Bit(
                1,
                ByteSource::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 1, D
            0x4A => Some(Instruction::Bit(
                1,
                ByteSource::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 1,E
            0x4B => Some(Instruction::Bit(
                1,
                ByteSource::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 1, H
            0x4C => Some(Instruction::Bit(
                1,
                ByteSource::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 1, L
            0x4D => Some(Instruction::Bit(
                1,
                ByteSource::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 1, (HL)
            0x4E => Some(Instruction::Bit(
                1,
                ByteSource::HL,
                Data::new_cb(2, 12, None, opcode),
            )),

            // BIT 1, A
            0x4F => Some(Instruction::Bit(
                1,
                ByteSource::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 2, B
            0x50 => Some(Instruction::Bit(
                2,
                ByteSource::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 2, C
            0x51 => Some(Instruction::Bit(
                2,
                ByteSource::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 2, D
            0x52 => Some(Instruction::Bit(
                2,
                ByteSource::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 2, E
            0x53 => Some(Instruction::Bit(
                2,
                ByteSource::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 2, H
            0x54 => Some(Instruction::Bit(
                2,
                ByteSource::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 2, L
            0x55 => Some(Instruction::Bit(
                2,
                ByteSource::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 2, (HL)
            0x56 => Some(Instruction::Bit(
                2,
                ByteSource::HL,
                Data::new_cb(2, 12, None, opcode),
            )),

            // BIT 2, A
            0x57 => Some(Instruction::Bit(
                2,
                ByteSource::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 3, B
            0x58 => Some(Instruction::Bit(
                3,
                ByteSource::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 3, C
            0x59 => Some(Instruction::Bit(
                3,
                ByteSource::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 3, D
            0x5A => Some(Instruction::Bit(
                3,
                ByteSource::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 3, E
            0x5B => Some(Instruction::Bit(
                3,
                ByteSource::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 3, H
            0x5C => Some(Instruction::Bit(
                3,
                ByteSource::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 3, L
            0x5D => Some(Instruction::Bit(
                3,
                ByteSource::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 3, (HL)
            0x5E => Some(Instruction::Bit(
                3,
                ByteSource::HL,
                Data::new_cb(2, 12, None, opcode),
            )),

            // BIT 3, A
            0x5F => Some(Instruction::Bit(
                3,
                ByteSource::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 4, B
            0x60 => Some(Instruction::Bit(
                4,
                ByteSource::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 4, C
            0x61 => Some(Instruction::Bit(
                4,
                ByteSource::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 4, D
            0x62 => Some(Instruction::Bit(
                4,
                ByteSource::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 4, E
            0x63 => Some(Instruction::Bit(
                4,
                ByteSource::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 4, H
            0x64 => Some(Instruction::Bit(
                4,
                ByteSource::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 4, L
            0x65 => Some(Instruction::Bit(
                4,
                ByteSource::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 4, (HL)
            0x66 => Some(Instruction::Bit(
                4,
                ByteSource::HL,
                Data::new_cb(2, 12, None, opcode),
            )),

            // BIT 4, A
            0x67 => Some(Instruction::Bit(
                4,
                ByteSource::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 5, B
            0x68 => Some(Instruction::Bit(
                5,
                ByteSource::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 5, C
            0x69 => Some(Instruction::Bit(
                5,
                ByteSource::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 5, D
            0x6A => Some(Instruction::Bit(
                5,
                ByteSource::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 5, E
            0x6B => Some(Instruction::Bit(
                5,
                ByteSource::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 5, H
            0x6C => Some(Instruction::Bit(
                5,
                ByteSource::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 5, L
            0x6D => Some(Instruction::Bit(
                5,
                ByteSource::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 5, (HL)
            0x6E => Some(Instruction::Bit(
                5,
                ByteSource::HL,
                Data::new_cb(2, 12, None, opcode),
            )),

            // BIT 5, A
            0x6F => Some(Instruction::Bit(
                5,
                ByteSource::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 6, B
            0x70 => Some(Instruction::Bit(
                6,
                ByteSource::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 6, C
            0x71 => Some(Instruction::Bit(
                6,
                ByteSource::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 6, D
            0x72 => Some(Instruction::Bit(
                6,
                ByteSource::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 6, E
            0x73 => Some(Instruction::Bit(
                6,
                ByteSource::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 6, H
            0x74 => Some(Instruction::Bit(
                6,
                ByteSource::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 6, L
            0x75 => Some(Instruction::Bit(
                6,
                ByteSource::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 6, (HL)
            0x76 => Some(Instruction::Bit(
                6,
                ByteSource::HL,
                Data::new_cb(2, 12, None, opcode),
            )),

            // BIT 6, A
            0x77 => Some(Instruction::Bit(
                6,
                ByteSource::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 7, B
            0x78 => Some(Instruction::Bit(
                7,
                ByteSource::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 7, C
            0x79 => Some(Instruction::Bit(
                7,
                ByteSource::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 7, D
            0x7A => Some(Instruction::Bit(
                7,
                ByteSource::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 7, E
            0x7B => Some(Instruction::Bit(
                7,
                ByteSource::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 7, H
            0x7C => Some(Instruction::Bit(
                7,
                ByteSource::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 7, L
            0x7D => Some(Instruction::Bit(
                7,
                ByteSource::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // BIT 7, (HL)
            0x7E => Some(Instruction::Bit(
                7,
                ByteSource::HL,
                Data::new_cb(2, 12, None, opcode),
            )),

            // BIT 7, A
            0x7F => Some(Instruction::Bit(
                7,
                ByteSource::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 0, B
            0x80 => Some(Instruction::Res(
                0,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 0, C
            0x81 => Some(Instruction::Res(
                0,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 0, D
            0x82 => Some(Instruction::Res(
                0,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 0, E
            0x83 => Some(Instruction::Res(
                0,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 0, H
            0x84 => Some(Instruction::Res(
                0,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 0, L
            0x85 => Some(Instruction::Res(
                0,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 0, (HL)
            0x86 => Some(Instruction::Res(
                0,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RES 0, A
            0x87 => Some(Instruction::Res(
                0,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 1, B
            0x88 => Some(Instruction::Res(
                1,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 1, C
            0x89 => Some(Instruction::Res(
                1,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 1, D
            0x8A => Some(Instruction::Res(
                1,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 1, E
            0x8B => Some(Instruction::Res(
                1,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 1, H
            0x8C => Some(Instruction::Res(
                1,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 1, L
            0x8D => Some(Instruction::Res(
                1,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 1, (HL)
            0x8E => Some(Instruction::Res(
                1,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RES 1, A
            0x8F => Some(Instruction::Res(
                1,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 2, B
            0x90 => Some(Instruction::Res(
                2,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 2, C
            0x91 => Some(Instruction::Res(
                2,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES RES 2, D
            0x92 => Some(Instruction::Res(
                2,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 2, E
            0x93 => Some(Instruction::Res(
                2,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 2, H
            0x94 => Some(Instruction::Res(
                2,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 2, L
            0x95 => Some(Instruction::Res(
                2,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 2, (HL)
            0x96 => Some(Instruction::Res(
                2,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RES 2, A
            0x97 => Some(Instruction::Res(
                2,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 3, B
            0x98 => Some(Instruction::Res(
                3,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 3, C
            0x99 => Some(Instruction::Res(
                3,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 3, D
            0x9A => Some(Instruction::Res(
                3,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 3, E
            0x9B => Some(Instruction::Res(
                3,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 3, H
            0x9C => Some(Instruction::Res(
                3,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 3, L
            0x9D => Some(Instruction::Res(
                3,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 3, (HL)
            0x9E => Some(Instruction::Res(
                3,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RES 3, A
            0x9F => Some(Instruction::Res(
                3,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 4, B
            0xA0 => Some(Instruction::Res(
                4,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 4, C
            0xA1 => Some(Instruction::Res(
                4,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 4, D
            0xA2 => Some(Instruction::Res(
                4,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 4, E
            0xA3 => Some(Instruction::Res(
                4,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 4, H
            0xA4 => Some(Instruction::Res(
                4,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 4, L
            0xA5 => Some(Instruction::Res(
                4,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 4, (HL)
            0xA6 => Some(Instruction::Res(
                4,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RES 4, A
            0xA7 => Some(Instruction::Res(
                4,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 5, B
            0xA8 => Some(Instruction::Res(
                5,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 5, C
            0xA9 => Some(Instruction::Res(
                5,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 5, D
            0xAA => Some(Instruction::Res(
                5,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 5, E
            0xAB => Some(Instruction::Res(
                5,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 5, H
            0xAC => Some(Instruction::Res(
                5,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 5, L
            0xAD => Some(Instruction::Res(
                5,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 5, (HL)
            0xAE => Some(Instruction::Res(
                5,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RES 5, A
            0xAF => Some(Instruction::Res(
                5,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 6, B
            0xB0 => Some(Instruction::Res(
                6,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 6, C
            0xB1 => Some(Instruction::Res(
                6,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 6, D
            0xB2 => Some(Instruction::Res(
                6,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 6, E
            0xB3 => Some(Instruction::Res(
                6,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 6, H
            0xB4 => Some(Instruction::Res(
                6,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 6, L
            0xB5 => Some(Instruction::Res(
                6,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 6, (HL)
            0xB6 => Some(Instruction::Res(
                6,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RES 6, A
            0xB7 => Some(Instruction::Res(
                6,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 7, B
            0xB8 => Some(Instruction::Res(
                7,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 7, C
            0xB9 => Some(Instruction::Res(
                7,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 7, D
            0xBA => Some(Instruction::Res(
                7,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 7, E
            0xBB => Some(Instruction::Res(
                7,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 7, H
            0xBC => Some(Instruction::Res(
                7,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 7, L
            0xBD => Some(Instruction::Res(
                7,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // RES 7, (HL)
            0xBE => Some(Instruction::Res(
                7,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // RES 7, A
            0xBF => Some(Instruction::Res(
                7,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 0, B
            0xC0 => Some(Instruction::Set(
                0,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 0, C
            0xC1 => Some(Instruction::Set(
                0,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 0, D
            0xC2 => Some(Instruction::Set(
                0,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 0, E
            0xC3 => Some(Instruction::Set(
                0,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 0, H
            0xC4 => Some(Instruction::Set(
                0,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 0, L
            0xC5 => Some(Instruction::Set(
                0,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 0, (HL)
            0xC6 => Some(Instruction::Set(
                0,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SET 0, A
            0xC7 => Some(Instruction::Set(
                0,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 1, B
            0xC8 => Some(Instruction::Set(
                1,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 1, C
            0xC9 => Some(Instruction::Set(
                1,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 1, D
            0xCA => Some(Instruction::Set(
                1,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 1, E
            0xCB => Some(Instruction::Set(
                1,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 1, H
            0xCC => Some(Instruction::Set(
                1,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 1, L
            0xCD => Some(Instruction::Set(
                1,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 1, (HL)
            0xCE => Some(Instruction::Set(
                1,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SET 1, A
            0xCF => Some(Instruction::Set(
                1,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 2, B
            0xD0 => Some(Instruction::Set(
                2,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 2, C
            0xD1 => Some(Instruction::Set(
                2,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 2, D
            0xD2 => Some(Instruction::Set(
                2,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 2, E
            0xD3 => Some(Instruction::Set(
                2,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 2, H
            0xD4 => Some(Instruction::Set(
                2,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 2, L
            0xD5 => Some(Instruction::Set(
                2,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 2, (HL)
            0xD6 => Some(Instruction::Set(
                2,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SET 2, A
            0xD7 => Some(Instruction::Set(
                2,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 3, B
            0xD8 => Some(Instruction::Set(
                3,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 3, C
            0xD9 => Some(Instruction::Set(
                3,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 3, D
            0xDA => Some(Instruction::Set(
                3,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 3, E
            0xDB => Some(Instruction::Set(
                3,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 3, H
            0xDC => Some(Instruction::Set(
                3,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 3, L
            0xDD => Some(Instruction::Set(
                3,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 3, (HL)
            0xDE => Some(Instruction::Set(
                3,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SET 3, A
            0xDF => Some(Instruction::Set(
                3,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 4, B
            0xE0 => Some(Instruction::Set(
                4,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 4, C
            0xE1 => Some(Instruction::Set(
                4,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 4, D
            0xE2 => Some(Instruction::Set(
                4,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 4, E
            0xE3 => Some(Instruction::Set(
                4,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 4, H
            0xE4 => Some(Instruction::Set(
                4,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 4,L
            0xE5 => Some(Instruction::Set(
                4,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 4, (HL)
            0xE6 => Some(Instruction::Set(
                4,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SET 4, A
            0xE7 => Some(Instruction::Set(
                4,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 5, B
            0xE8 => Some(Instruction::Set(
                5,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 5, C
            0xE9 => Some(Instruction::Set(
                5,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 5, D
            0xEA => Some(Instruction::Set(
                5,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 5, E
            0xEB => Some(Instruction::Set(
                5,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 5, H
            0xEC => Some(Instruction::Set(
                5,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 5, L
            0xED => Some(Instruction::Set(
                5,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 5,(HL)
            0xEE => Some(Instruction::Set(
                5,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SET 5, A
            0xEF => Some(Instruction::Set(
                5,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 6, B
            0xF0 => Some(Instruction::Set(
                6,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 6, C
            0xF1 => Some(Instruction::Set(
                6,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 6, D
            0xF2 => Some(Instruction::Set(
                6,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 6, E
            0xF3 => Some(Instruction::Set(
                6,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 6, H
            0xF4 => Some(Instruction::Set(
                6,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 6, L
            0xF5 => Some(Instruction::Set(
                6,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 6, (HL)
            0xF6 => Some(Instruction::Set(
                6,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SET 6, A
            0xF7 => Some(Instruction::Set(
                6,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 7, B
            0xF8 => Some(Instruction::Set(
                7,
                ByteTarget::Register(R::B),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 7, C
            0xF9 => Some(Instruction::Set(
                7,
                ByteTarget::Register(R::C),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 7, D
            0xFA => Some(Instruction::Set(
                7,
                ByteTarget::Register(R::D),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 7, E
            0xFB => Some(Instruction::Set(
                7,
                ByteTarget::Register(R::E),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 7, H
            0xFC => Some(Instruction::Set(
                7,
                ByteTarget::Register(R::H),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 7, L
            0xFD => Some(Instruction::Set(
                7,
                ByteTarget::Register(R::L),
                Data::new_cb(2, 8, None, opcode),
            )),

            // SET 7, (HL)
            0xFE => Some(Instruction::Set(
                7,
                ByteTarget::HL,
                Data::new_cb(2, 16, None, opcode),
            )),

            // SET 7, A
            0xFF => Some(Instruction::Set(
                7,
                ByteTarget::Register(R::A),
                Data::new_cb(2, 8, None, opcode),
            )),
        }
    }
}
