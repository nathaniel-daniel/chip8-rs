pub mod instruction;

pub use crate::instruction::Instruction;
use rand::{
    rngs::OsRng,
    Rng,
};
use std::fmt;

const FONT: &[u8] = &[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub const MEMORY_SIZE: usize = 4096;
pub const NUM_REGISTERS: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const NUM_KEYS: usize = 16;
pub const GFX_WIDTH: usize = 64;
pub const GFX_HEIGHT: usize = 64;
pub const GFX_SIZE: usize = GFX_WIDTH * GFX_HEIGHT;

pub const MEMORY_START: usize = 0x200;
pub const OPCODE_SIZE: u16 = 2;
pub const FLAG_REG: u8 = 0xF;

#[derive(Debug)]
pub enum Chip8Error {
    InvalidProgramSize(usize),
    UnknownInstruction(Instruction),
    InvalidReg(u8),
    StackUnderflow,
    StackOverflow,
    ProgramCounterOutOfBounds(u16),
}

pub type Chip8Result<T> = Result<T, Chip8Error>;

pub struct Chip8 {
    /// Memory
    memory: [u8; MEMORY_SIZE],

    /// Registers
    v: [u8; NUM_REGISTERS],

    i: u16,

    /// Program counter
    pc: u16,

    /// Return Address Stack
    stack: [u16; STACK_SIZE],

    /// Stack Pointer
    sp: u8,

    /// GFX memory
    pub gfx: [bool; GFX_SIZE],

    delay_timer: u8,
    sound_timer: u8,
    draw_flag: bool,
    keys: [bool; NUM_KEYS],
    key_pressed: Option<u8>,
}

impl Chip8 {
    /// Create a new emulator
    pub fn new() -> Self {
        Chip8 {
            memory: [0; MEMORY_SIZE],
            v: [0; NUM_REGISTERS],
            i: 0,
            pc: MEMORY_START as u16,
            stack: [0; STACK_SIZE],
            sp: 0,
            gfx: [false; GFX_SIZE],
            delay_timer: 0,
            sound_timer: 0,
            draw_flag: false,
            keys: [false; NUM_KEYS],
            key_pressed: None,
        }
    }

    /// Reset the chip8 state
    pub fn init(&mut self) {
        self.i = 0;
        self.memory = [0; MEMORY_SIZE];
        self.v = [0; NUM_REGISTERS];
        self.pc = MEMORY_START as u16;
        self.stack = [0; STACK_SIZE];
        self.sp = 0;
        self.gfx = [false; GFX_SIZE];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.draw_flag = false;
        self.keys = [false; NUM_KEYS];
        self.key_pressed = None;

        for (i, &el) in FONT.iter().enumerate() {
            self.memory[i] = el;
        }
    }

    /// Load a rom
    pub fn load(&mut self, data: &[u8]) -> Chip8Result<()> {
        let data_len = data.len();
        if data_len > self.memory.len() {
            return Err(Chip8Error::InvalidProgramSize(data_len));
        }

        self.memory[MEMORY_START..(data.len() + MEMORY_START)].clone_from_slice(&data[..]);

        Ok(())
    }

    /// Execute 1 cycle
    pub fn cycle(&mut self) -> Chip8Result<Instruction> {
        if self.pc >= MEMORY_SIZE as u16 {
            return Err(Chip8Error::ProgramCounterOutOfBounds(self.pc));
        }

        let op1 = self.memory[self.pc as usize] as u16;
        let op2 = self.memory[self.pc as usize + 1] as u16;
        let op = (op1 << 8) + op2;
        let op = Instruction::from(op);

        match op {
            Instruction::ClearDisplay => {
                self.gfx.iter_mut().for_each(|el| *el = false);
                self.draw_flag = true;
                self.pc += OPCODE_SIZE;
            }
            Instruction::Return => {
                self.pc = self.pop_stack()?;
            }
            Instruction::SkipEqualConst(x, val) => {
                self.pc += if self.read_reg(x)? == val {
                    OPCODE_SIZE * 2
                } else {
                    OPCODE_SIZE
                }
            }
            Instruction::SkipNotEqualConst(x, val) => {
                self.pc += if self.read_reg(x)? != val {
                    OPCODE_SIZE * 2
                } else {
                    OPCODE_SIZE
                }
            }
            Instruction::SkipEqual(x, y) => {
                let x = self.read_reg(x)?;
                let y = self.read_reg(y)?;

                if x == y {
                    self.pc += OPCODE_SIZE * 2;
                } else {
                    self.pc += OPCODE_SIZE;
                }
            }
            Instruction::Jump(addr) => {
                self.pc = addr;
            }
            Instruction::Call(addr) => {
                self.push_stack(self.pc + OPCODE_SIZE)?;
                self.pc = addr;
            }
            Instruction::SetVConst(x, val) => {
                self.write_reg(x, val)?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::AddVConst(x, val) => {
                let reg_x = self.read_reg(x)?;
                self.write_reg(x, reg_x.wrapping_add(val))?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::SetV(x, y) => {
                let reg_y = self.read_reg(y)?;
                self.write_reg(x, reg_y)?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::Or(x, y) => {
                let reg_x = self.read_reg(x)?;
                let reg_y = self.read_reg(y)?;
                self.write_reg(x, reg_x | reg_y)?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::And(x, y) => {
                let reg_x = self.read_reg(x)?;
                let reg_y = self.read_reg(y)?;
                self.write_reg(x, reg_x & reg_y)?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::Xor(x, y) => {
                let reg_x = self.read_reg(x)?;
                let reg_y = self.read_reg(y)?;
                self.write_reg(x, reg_x ^ reg_y)?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::Add(x, y) => {
                let reg_x = self.read_reg(x)?;
                let reg_y = self.read_reg(y)?;
                let res = u16::from(reg_x) + u16::from(reg_y);
                self.write_reg(FLAG_REG, if res > u16::from(u8::MAX) { 1 } else { 0 })?;
                self.write_reg(x, (res & u16::from(u8::MAX)) as u8)?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::Sub(x, y) => {
                let reg_x = self.read_reg(x)?;
                let reg_y = self.read_reg(y)?;
                self.write_reg(FLAG_REG, if reg_x > reg_y { 1 } else { 0 })?;
                self.write_reg(x, reg_x.wrapping_sub(reg_y))?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::ShiftRight(x) => {
                let reg_x = self.read_reg(x)?;
                self.write_reg(FLAG_REG, reg_x & 0x1)?;
                self.write_reg(x, reg_x >> 1)?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::SubN(x, y) => {
                let reg_x = self.read_reg(x)?;
                let reg_y = self.read_reg(y)?;
                self.write_reg(FLAG_REG, if reg_y > reg_x { 1 } else { 0 })?;
                self.write_reg(x, reg_y.wrapping_sub(reg_x))?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::ShiftLeft(x) => {
                let reg_x = self.read_reg(x)?;
                self.write_reg(FLAG_REG, (reg_x & 0b10000000) >> 7)?;
                self.write_reg(x, reg_x << 1)?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::SkipNotEqual(x, y) => {
                let reg_x = self.read_reg(x)?;
                let reg_y = self.read_reg(y)?;
                self.pc += if reg_x != reg_y {
                    2 * OPCODE_SIZE
                } else {
                    OPCODE_SIZE
                };
            }
            Instruction::SetI(val) => {
                self.i = val;
                self.pc += OPCODE_SIZE;
            }
            Instruction::Rand(x, val) => {
                self.write_reg(x, OsRng.gen::<u8>() & val)?;
                self.pc += OPCODE_SIZE;
            }
            Instruction::Draw(x, y, n) => {
                let reg_x = self.read_reg(x)?;
                let reg_y = self.read_reg(y)?;
                self.write_reg(FLAG_REG, 0)?;

                for y in 0..n {
                    let pix_row = self.memory[self.i as usize + y as usize];
                    for x in 0..8 {
                        let pix = pix_row & (0x01 << (7 - x)) != 0;
                        let gfx_index =
                            (reg_x as usize + x as usize + ((y + reg_y) as usize * 64)) % (32 * 64);
                        if self.gfx[gfx_index] && pix {
                            self.write_reg(FLAG_REG, 1)?;
                        }
                        self.gfx[gfx_index] ^= pix;
                    }
                }

                self.draw_flag = true;
                self.pc += OPCODE_SIZE;
            }
            Instruction::SkipPressed(x) => {
                let reg_x = self.read_reg(x)?;
                self.pc += if self.keys[reg_x as usize] {
                    2 * OPCODE_SIZE
                } else {
                    OPCODE_SIZE
                }
            }
            Instruction::SkipNotPressed(x) => {
                let reg_x = self.read_reg(x)?;
                self.pc += if self.keys[reg_x as usize] {
                    OPCODE_SIZE
                } else {
                    2 * OPCODE_SIZE
                }
            }
            Instruction::LoadDelay(reg) => {
                self.v[reg as usize] = self.delay_timer;
                self.pc += OPCODE_SIZE;
            }
            Instruction::HaltUntilPressed(reg) => {
                if let Some(code) = self.key_pressed {
                    self.v[reg as usize] = code;
                    self.pc += OPCODE_SIZE;
                }
            }
            Instruction::SetDelay(x) => {
                let reg_x = self.read_reg(x)?;
                self.delay_timer = reg_x;
                self.pc += OPCODE_SIZE;
            }
            Instruction::SetSound(x) => {
                let reg_x = self.read_reg(x)?;
                self.sound_timer = reg_x;
                self.pc += OPCODE_SIZE;
            }
            Instruction::AddI(reg) => {
                self.i += u16::from(self.read_reg(reg)?);
                self.pc += OPCODE_SIZE;
            }
            Instruction::LoadFont(reg) => {
                self.i = u16::from(self.read_reg(reg)?) * 5;
                self.pc += OPCODE_SIZE;
            }
            Instruction::StoreBcd(x) => {
                let reg_x = self.read_reg(x)?;
                self.memory[self.i as usize] = reg_x / 100;
                self.memory[self.i as usize + 1] = (reg_x / 10) % 10;
                self.memory[self.i as usize + 2] = (reg_x % 100) % 10;
                self.pc += OPCODE_SIZE;
            }
            Instruction::StoreV(x) => {
                for i in 0..x + 1 {
                    self.memory[self.i as usize + usize::from(i)] = self.read_reg(i)?;
                }
                self.pc += OPCODE_SIZE;
            }
            Instruction::LoadV(x) => {
                for i in 0..x + 1 {
                    self.write_reg(i, self.memory[self.i as usize + usize::from(i)])?;
                }
                self.pc += OPCODE_SIZE;
            }
            Instruction::Unknown(_) => {
                return Err(Chip8Error::UnknownInstruction(op));
            }
        }

        self.key_pressed = None;

        Ok(op)
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer != 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer != 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn set_key(&mut self, key: usize, data: bool) {
        self.keys[key] = data;
        if data {
            self.key_pressed = Some(key as u8);
        }
    }

    #[inline]
    fn push_stack(&mut self, data: u16) -> Chip8Result<()> {
        self.stack[self.sp as usize] = data;
        self.sp += 1;

        if self.sp == STACK_SIZE as u8 {
            return Err(Chip8Error::StackOverflow);
        }

        Ok(())
    }

    #[inline]
    fn pop_stack(&mut self) -> Chip8Result<u16> {
        if self.sp == 0 {
            return Err(Chip8Error::StackUnderflow);
        }

        self.sp -= 1;
        let ret = self.stack[self.sp as usize];
        self.stack[self.sp as usize] = 0;

        Ok(ret)
    }

    #[inline]
    fn read_reg(&mut self, reg: u8) -> Chip8Result<u8> {
        self.v
            .get(usize::from(reg))
            .copied()
            .ok_or(Chip8Error::InvalidReg(reg))
    }

    #[inline]
    fn write_reg(&mut self, reg: u8, value: u8) -> Chip8Result<()> {
        *self
            .v
            .get_mut(usize::from(reg))
            .ok_or(Chip8Error::InvalidReg(reg))? = value;
        Ok(())
    }
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Chip8")?;
        writeln!(f, "PC: {}", self.pc)?;
        writeln!(f, "Stack: {:?}", self.stack)?;
        writeln!(f, "SP: {}", self.sp)?;
        writeln!(f, "V: {:?}", self.v)?;
        writeln!(f, "I: {:?}", self.i)?;
        writeln!(f, "Delay timer: {}", self.delay_timer)?;
        write!(f, "Sound timer: {}", self.sound_timer)
    }
}
