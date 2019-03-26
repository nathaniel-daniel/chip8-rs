extern crate rand;
use rand::{
    thread_rng,
    Rng,
};
use std::{
    fmt,
    u16,
    u8,
};

const FONT: &'static [u8] = &[
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

#[derive(Debug)]
pub enum Chip8Error {}

pub type Chip8Result<T> = Result<T, Chip8Error>;

pub struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: [u16; 16],
    sp: u8,
    pub gfx: [bool; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    draw_flag: bool,
    keys: [bool; 16],
    key_pressed: Option<u8>,
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            gfx: [false; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            draw_flag: false,
            keys: [false; 16],
            key_pressed: None,
        }
    }

    pub fn init(&mut self) {
        self.i = 0;
        self.memory = [0; 4096];
        self.v = [0; 16];
        self.pc = 0x200;
        self.stack = [0; 16];
        self.sp = 0;
        self.gfx = [false; 64 * 32];
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.draw_flag = false;
        self.keys = [false; 16];
        self.key_pressed = None;

        for (i, &el) in FONT.iter().enumerate() {
            self.memory[i] = el;
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        if data.len() > self.memory.len() {
            panic!("Error: Program larger than memory")
        }

        for i in 0..data.len() {
            self.memory[i + 0x200] = data[i];
        }
    }

    pub fn cycle(&mut self) -> Chip8Result<Instruction> {
        let raw_op = ((self.memory[self.pc as usize] as u16) << 8)
            + self.memory[self.pc as usize + 1] as u16;
        let op = raw_op.into();
        match op {
            Instruction::ClearDisplay => {
                self.gfx.iter_mut().for_each(|el| *el = false);
                self.draw_flag = true;
                self.pc += 2;
            }
            Instruction::Return => {
                self.pc = self.pop_stack();
            }
            Instruction::SkipEqualConst(reg, val) => {
                self.pc += if self.v[reg as usize] == val { 4 } else { 2 }
            }
            Instruction::SkipNotEqualConst(reg, val) => {
                self.pc += if self.v[reg as usize] != val { 4 } else { 2 }
            }
            Instruction::Jump(addr) => {
                self.pc = addr;
            }
            Instruction::Call(addr) => {
                self.push_stack(self.pc + 2);
                self.pc = addr;
            }
            Instruction::SetVConst(reg, val) => {
                self.v[reg as usize] = val as u8;
                self.pc += 2;
            }
            Instruction::AddVConst(reg, val) => {
                self.v[reg as usize] = self.get_reg(reg).overflowing_add(val).0;
                self.pc += 2;
            }
            Instruction::SetV(reg_x, reg_y) => {
                self.v[reg_x as usize] = self.v[reg_y as usize];
                self.pc += 2;
            }
            Instruction::Or(reg_x, reg_y) => {
                self.v[reg_x as usize] |= self.v[reg_y as usize];
                self.pc += 2;
            }
            Instruction::And(reg_x, reg_y) => {
                self.v[reg_x as usize] &= self.v[reg_y as usize];
                self.pc += 2;
            }
            Instruction::Xor(reg_x, reg_y) => {
                self.v[reg_x as usize] ^= self.get_reg(reg_y);
                self.pc += 2;
            }
            Instruction::Add(reg_x, reg_y) => {
                let res = self.v[reg_x as usize] as u16 + self.v[reg_y as usize] as u16;
                self.v[0xF] = if res > std::u8::MAX as u16 { 1 } else { 0 };
                self.v[reg_x as usize] = (res & 0xFF) as u8;
                self.pc += 2;
            }
            Instruction::Sub(reg_x, reg_y) => {
                self.v[0xF] = if reg_x > reg_y {
                    self.v[reg_x as usize] -= self.v[reg_y as usize];
                    1
                } else {
                    //panic!("{} {}", self.v[reg_x as usize], self.v[reg_y as usize]);
                    let x = self.v[reg_x as usize];
                    let y = self.v[reg_y as usize];
                    self.v[reg_x as usize] = x.overflowing_sub(y).0;
                    0
                };

                self.pc += 2;
            }
            Instruction::ShiftRight(reg) => {
                self.v[0xF] = reg & 0x80;
                self.v[reg as usize] <<= 1;
                self.pc += 2;
            }
            Instruction::ShiftLeft(reg) => {
                self.v[0xF] = reg & 0x1;
                self.v[reg as usize] >>= 1;
                self.pc += 2;
            }
            Instruction::SkipNotEqual(reg_x, reg_y) => {
                self.pc += if self.get_reg(reg_x) != self.get_reg(reg_y) {
                    4
                } else {
                    2
                }
            }
            Instruction::SetI(val) => {
                self.i = val;
                self.pc += 2;
            }
            Instruction::Rand(reg, val) => {
                self.v[reg as usize] = thread_rng().gen::<u8>() & val;
                self.pc += 2;
            }
            Instruction::Draw(reg_x, reg_y, n) => {
                let reg_x = self.v[reg_x as usize];
                let reg_y = self.v[reg_y as usize];
                self.v[0xF] = 0;

                for y in 0..n {
                    let pix_row = self.memory[self.i as usize + y as usize];
                    for x in 0..8 {
                        let pix = pix_row & (0x01 << (7 - x)) != 0;
                        let gfx_index =
                            (reg_x as usize + x as usize + ((y + reg_y) as usize * 64)) % (32 * 64);
                        if self.gfx[gfx_index] && pix {
                            self.v[0xF] = 1;
                        }
                        self.gfx[gfx_index] ^= pix;
                    }
                }

                self.draw_flag = true;
                self.pc += 2;
            }
            Instruction::SkipPressed(reg) => {
                self.pc += if self.keys[self.v[reg as usize] as usize] {
                    4
                } else {
                    2
                }
            }
            Instruction::SkipNotPressed(reg) => {
                self.pc += if self.keys[self.v[reg as usize] as usize] {
                    2
                } else {
                    4
                }
            }
            Instruction::LoadDelay(reg) => {
                self.v[reg as usize] = self.delay_timer;
                self.pc += 2;
            }
            Instruction::HaltUntilPressed(reg) => {
                if let Some(code) = self.key_pressed {
                    self.v[reg as usize] = code;
                    self.pc += 2;
                }
            }
            Instruction::SetDelay(reg) => {
                self.delay_timer = self.v[reg as usize];
                self.pc += 2;
            }
            Instruction::SetSound(reg) => {
                self.sound_timer = self.v[reg as usize];
                self.pc += 2;
            }
            Instruction::AddI(reg) => {
                self.i += self.get_reg(reg) as u16;
                self.pc += 2;
            }
            Instruction::LoadFont(reg) => {
                self.i = self.get_reg(reg) as u16 * 5;
                self.pc += 2;
            }
            Instruction::StoreBcd(reg) => {
                let reg = self.v[reg as usize];
                let ones = (reg % 100) % 10; //100 unecessary? its clear though?
                let tens = (reg / 10) % 10;
                let hundreds = reg / 100; //u8, max size below 1000
                self.memory[self.i as usize + 1] = hundreds;
                self.memory[self.i as usize + 1] = tens;
                self.memory[self.i as usize + 1] = ones;
                self.pc += 2;
            }
            Instruction::StoreV(reg) => {
                for i in 0..=reg as usize {
                    self.memory[self.i as usize + i] = self.v[i];
                }
                self.pc += 2;
            }
            Instruction::LoadV(reg) => {
                for i in 0..=reg as usize {
                    self.v[i] = self.memory[self.i as usize + i];
                }
                self.pc += 2;
            }
            Instruction::Unknown(val) => {
                println!("Unknown: 0x{:04X}", val);
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

    fn push_stack(&mut self, data: u16) {
        self.stack[self.sp as usize] = data;
        self.sp += 1;
    }

    fn pop_stack(&mut self) -> u16 {
        self.sp -= 1;
        let ret = self.stack[self.sp as usize];
        self.stack[self.sp as usize] = 0;
        return ret;
    }

    fn get_reg(&mut self, reg: u8) -> u8 {
        self.v[reg as usize]
    }
}

impl fmt::Display for Chip8 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Chip8\n")?;
        write!(f, "PC: {}\n", self.pc)?;
        write!(f, "Stack: {:?}\n", self.stack)?;
        write!(f, "SP: {}\n", self.sp)?;
        write!(f, "V: {:?}\n", self.v)?;
        write!(f, "I: {:?}\n", self.i)?;
        write!(f, "Delay timer: {}", self.delay_timer)?;
        write!(f, "Sound timer: {}", self.sound_timer)
    }
}

#[derive(Debug)]
pub enum Instruction {
    ClearDisplay,
    Return,
    Jump(u16),
    Call(u16),
    SkipEqualConst(u8, u8),
    SkipNotEqualConst(u8, u8),
    SetVConst(u8, u8),
    AddVConst(u8, u8),

    //Math
    SetV(u8, u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    Add(u8, u8),
    Sub(u8, u8),
    ShiftRight(u8),
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
        match n & 0xF000 {
            0x0000 => match n & 0x0FFF {
                0x00E0 => Instruction::ClearDisplay,
                0x00EE => Instruction::Return,
                _ => Instruction::Unknown(n),
            },
            0x1000 => Instruction::Jump(n & 0xFFF),
            0x2000 => Instruction::Call(n & 0xFFF),
            0x3000 => Instruction::SkipEqualConst(((n & 0x0F00) >> 8) as u8, (n & 0xFF) as u8),
            0x4000 => Instruction::SkipNotEqualConst(((n & 0x0F00) >> 8) as u8, (n & 0xFF) as u8),
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
                0x33 => Instruction::StoreBcd(((n & 0x0F00) >> 8) as u8),
                0x55 => Instruction::StoreV(((n & 0x0F00) >> 8) as u8),
                0x65 => Instruction::LoadV(((n & 0x0F00) >> 8) as u8),
                _ => Instruction::Unknown(n),
            },
            _ => Instruction::Unknown(n),
        }
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            _ => write!(f, "{:?}", self),
        }
    }
}
