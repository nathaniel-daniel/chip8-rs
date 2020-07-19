#[derive(Debug)]
pub enum Instruction {
    ClearDisplay,
    Return,
    Jump(u16),
    Call(u16),
    SkipEqualConst(u8, u8),
    SkipNotEqualConst(u8, u8),
    SkipEqual(u8, u8),
    SetVConst(u8, u8),
    AddVConst(u8, u8),

    // Math
    SetV(u8, u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    Add(u8, u8),
    Sub(u8, u8),
    ShiftRight(u8),
    SubN(u8, u8),
    ShiftLeft(u8),

    SkipNotEqual(u8, u8),
    SetI(u16),
    Rand(u8, u8),
    Draw(u8, u8, u8),
    SkipPressed(u8),
    SkipNotPressed(u8),
    LoadDelay(u8),
    HaltUntilPressed(u8),
    SetDelay(u8),
    SetSound(u8),
    AddI(u8),
    LoadFont(u8),
    StoreBcd(u8),
    StoreV(u8),
    LoadV(u8),

    Unknown(u16),
}

impl From<u16> for Instruction {
    fn from(n: u16) -> Self {
        let nnn = n & 0xFFF;
        let x = ((n & 0x0F00) >> 8) as u8;
        let y = ((n & 0x00F0) >> 4) as u8;

        match n & 0xF000 {
            0x0000 => match n & 0x0FFF {
                0x00E0 => Instruction::ClearDisplay,
                0x00EE => Instruction::Return,
                _ => Instruction::Unknown(n),
            },
            0x1000 => Instruction::Jump(nnn),
            0x2000 => Instruction::Call(nnn),
            0x3000 => Instruction::SkipEqualConst(x, (n & 0xFF) as u8),
            0x4000 => Instruction::SkipNotEqualConst(x, (n & 0xFF) as u8),
            0x5000 => match n & 0x000F {
                0x0 => Instruction::SkipEqual(x, y),
                _ => Instruction::Unknown(n),
            },
            0x6000 => Instruction::SetVConst(((n & 0x0F00) >> 8) as u8, (n & 0xFF) as u8),
            0x7000 => Instruction::AddVConst(((n & 0x0F00) >> 8) as u8, (n & 0xFF) as u8),
            0x8000 => match n & 0xF {
                0x0 => Instruction::SetV(((n & 0xF00) >> 8) as u8, ((n & 0x0F0) >> 4) as u8),
                0x1 => Instruction::Or(((n & 0xF00) >> 8) as u8, ((n & 0x0F0) >> 4) as u8),
                0x2 => Instruction::And(((n & 0xF00) >> 8) as u8, ((n & 0x0F0) >> 4) as u8),
                0x3 => Instruction::Xor(((n & 0xF00) >> 8) as u8, ((n & 0x0F0) >> 4) as u8),
                0x4 => Instruction::Add(((n & 0xF00) >> 8) as u8, ((n & 0x0F0) >> 4) as u8),
                0x5 => Instruction::Sub(((n & 0xF00) >> 8) as u8, ((n & 0x0F0) >> 4) as u8),
                0x6 => Instruction::ShiftRight(((n & 0xF00) >> 8) as u8),
                0x7 => Instruction::SubN(x, y),
                0xE => Instruction::ShiftLeft(((n & 0xF00) >> 8) as u8),
                _ => Instruction::Unknown(n),
            },
            0x9000 => Instruction::SkipNotEqual(((n & 0xF00) >> 8) as u8, ((n & 0x0F0) >> 4) as u8),
            0xA000 => Instruction::SetI(n & 0x0FFF),
            0xC000 => Instruction::Rand(((n & 0x0F00) >> 8) as u8, (n & 0xFF) as u8),
            0xD000 => Instruction::Draw(
                ((n & 0x0F00) >> 8) as u8,
                ((n & 0x00F0) >> 4) as u8,
                (n & 0x000F) as u8,
            ),
            0xE000 => match n & 0xFF {
                0x9E => Instruction::SkipPressed(((n & 0x0F00) >> 8) as u8),
                0xA1 => Instruction::SkipNotPressed(((n & 0x0F00) >> 8) as u8),
                _ => Instruction::Unknown(n),
            },
            0xF000 => match n & 0xFF {
                0x07 => Instruction::LoadDelay(((n & 0x0F00) >> 8) as u8),
                0x0A => Instruction::HaltUntilPressed(((n & 0x0F00) >> 8) as u8),
                0x15 => Instruction::SetDelay(((n & 0x0F00) >> 8) as u8),
                0x18 => Instruction::SetSound(((n & 0x0F00) >> 8) as u8),
                0x1E => Instruction::AddI(((n & 0x0F00) >> 8) as u8),
                0x29 => Instruction::LoadFont(((n & 0x0F00) >> 8) as u8),
                0x33 => Instruction::StoreBcd(x),
                0x55 => Instruction::StoreV(((n & 0x0F00) >> 8) as u8),
                0x65 => Instruction::LoadV(((n & 0x0F00) >> 8) as u8),
                _ => Instruction::Unknown(n),
            },
            _ => Instruction::Unknown(n),
        }
    }
}

// TODO: Consider spitting out assembly-like stuff
#[allow(clippy::match_single_binding)]
impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            _ => write!(f, "{:?}", self),
        }
    }
}
