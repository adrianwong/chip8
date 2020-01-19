use std::fs::File;
use std::io;
use std::io::prelude::*;

#[derive(Debug)]
pub struct Chip8 {
    memory: Vec<u8>,     // 4,096 bytes of RAM
    v: Vec<u8>,          // 16 general-purpose registers
    i: u16,              // 1 I-register
    delay_timer: u8,     // Decrements at a rate of 60Hz
    sound_timer: u8,     // Decrements at a rate of 60Hz
    pc: u16,             // Program counter
    sp: u8,              // Stack pointer
    stack: Vec<u16>,     // 16 stack levels
    keyboard: Vec<bool>, // 16-key hexadecimal keypad
    display: Vec<bool>,  // 64 x 32 monochrome display
}

// Hexadecimal sprites. Stored in area of RAM reserved for interpreter
const HEX_SPRITES: &[u8; 80] = &[
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

pub const DISPLAY_W: usize = 64;
pub const DISPLAY_H: usize = 32;

impl Chip8 {
    fn init() -> Chip8 {
        let mut memory = HEX_SPRITES.to_vec();
        memory.resize(4096, 0);

        Chip8 {
            memory: memory,
            v: vec![0; 16],
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: vec![0; 16],
            keyboard: vec![false; 16],
            display: vec![false; DISPLAY_W * DISPLAY_H],
        }
    }

    pub fn load_rom(fname: &str) -> Result<Chip8, io::Error> {
        let mut chip8 = Chip8::init();

        let mut f = File::open(fname)?;
        let mut buf = Vec::new();

        f.read_to_end(&mut buf)?;
        if buf.len() > 4096 - 0x200 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "ROM too big for RAM",
            ))
        } else {
            chip8.memory[0x200..(0x200 + buf.len())].copy_from_slice(&buf[..]);
            Ok(chip8)
        }
    }

    pub fn display(&self) -> &[bool] {
        &self.display[..]
    }

    pub fn set_key(&mut self, key: u8) {
        self.keyboard[key as usize] = true;
    }

    pub fn reset_keys(&mut self) {
        for key in &mut self.keyboard {
            *key = false
        }
    }

    pub fn execute_opcode(&mut self) {
        // Instructions are 2 bytes long and are stored most
        // significant byte first
        let hi = self.memory[self.pc as usize] as u16;
        let lo = self.memory[(self.pc as usize) + 1] as u16;
        let opcode = hi << 8 | lo;

        self.execute_opcode_internal(opcode);
    }

    fn execute_opcode_internal(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => match opcode {
                0x00E0 => self.cls(),
                0x00EE => self.ret(),
                _ => panic!("Unknown opcode: {:X?}", opcode),
            },
            0x1000 => self.jp1(opcode),
            0x2000 => self.call(opcode),
            0x3000 => self.se1(opcode),
            0x4000 => self.sne1(opcode),
            0x5000 => self.se2(opcode),
            0x6000 => self.ld01(opcode),
            0x7000 => self.add1(opcode),
            0x8000 => match opcode & 0x000F {
                0x0000 => self.ld02(opcode),
                0x0001 => self.or(opcode),
                0x0002 => self.and(opcode),
                0x0003 => self.xor(opcode),
                0x0004 => self.add2(opcode),
                0x0005 => self.sub(opcode),
                0x0006 => self.shr(opcode),
                0x0007 => self.subn(opcode),
                0x000E => self.shl(opcode),
                _ => panic!("Unknown opcode: {:X?}", opcode),
            },
            0x9000 => self.sne2(opcode),
            0xA000 => self.ld03(opcode),
            0xB000 => self.jp2(opcode),
            0xC000 => self.rnd(opcode),
            0xD000 => self.drw(opcode),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.skp(opcode),
                0x00A1 => self.sknp(opcode),
                _ => panic!("Unknown opcode: {:X?}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => self.ld04(opcode),
                0x000A => self.ld05(opcode),
                0x0015 => self.ld06(opcode),
                0x0018 => self.ld07(opcode),
                0x001E => self.add3(opcode),
                0x0029 => self.ld08(opcode),
                0x0033 => self.ld09(opcode),
                0x0055 => self.ld10(opcode),
                0x0065 => self.ld11(opcode),
                _ => panic!("Unknown opcode: {:X?}", opcode),
            },
            _ => panic!("Unknown opcode: {:X?}", opcode),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    // 00E0 : Clear the display
    fn cls(&mut self) {
        for displayed in &mut self.display {
            *displayed = false;
        }
        self.pc += 2;
    }

    // 00EE : Return from a subroutine
    fn ret(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
        self.pc += 2;
    }

    // 1nnn : Jump to location nnn
    fn jp1(&mut self, opcode: u16) {
        let nnn = opcode & 0x0FFF;
        self.pc = nnn;
    }

    // 2nnn : Call subroutine at nnn
    fn call(&mut self, opcode: u16) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = opcode & 0x0FFF;
    }

    // 3xkk : Skip next instruction if Vx == kk
    fn se1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;

        self.pc += if self.v[x] == kk { 4 } else { 2 };
    }

    // 4xkk : Skip next instruction if Vx != kk
    fn sne1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;

        self.pc += if self.v[x] != kk { 4 } else { 2 };
    }

    // 5xy0 : Skip next instruction if Vx == Vy
    fn se2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        self.pc += if self.v[x] == self.v[y] { 4 } else { 2 };
    }

    // 6xkk : Set Vx = kk
    fn ld01(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;

        self.v[x] = kk;
        self.pc += 2;
    }

    // 7xkk : Set Vx = Vx + kk
    fn add1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;

        self.v[x] += kk;
        self.pc += 2;
    }

    // 8xy0 : Set Vx = Vy
    fn ld02(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        self.v[x] = self.v[y];
        self.pc += 2;
    }

    // 8xy1 : Set Vx = Vx OR Vy
    fn or(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        self.v[x] |= self.v[y];
        self.pc += 2;
    }

    // 8xy2 : Set Vx = Vx AND Vy
    fn and(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        self.v[x] &= self.v[y];
        self.pc += 2;
    }

    // 8xy3 : Set Vx = Vx XOR Vy
    fn xor(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        self.v[x] ^= self.v[y];
        self.pc += 2;
    }

    // 8xy4 : Set Vx = Vx + Vy, set VF = carry
    fn add2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        let res = (self.v[x] as u16) + (self.v[y] as u16);

        self.v[0xF] = if (res & 0xFF00) > 0 { 1 } else { 0 };
        self.v[x] = (res & 0x00FF) as u8;
        self.pc += 2;
    }

    // 8xy5 : Set Vx = Vx - Vy, set VF = NOT borrow
    fn sub(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        let res = (self.v[x] as i16) - (self.v[y] as i16);

        self.v[0xF] = if res > 0 { 1 } else { 0 };
        self.v[x] = (res & 0x00FF) as u8;
        self.pc += 2;
    }

    // 8xy6 : Set Vx = Vx SHR 1
    fn shr(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        self.v[0xF] = self.v[x] & 0x01;
        self.v[x] >>= 1;
        self.pc += 2;
    }

    // 8xy7 : Set Vx = Vy - Vx, set VF = NOT borrow
    fn subn(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        let res = (self.v[y] as i16) - (self.v[x] as i16);

        self.v[0xF] = if res > 0 { 1 } else { 0 };
        self.v[x] = (res & 0x00FF) as u8;
        self.pc += 2;
    }

    // 8xyE : Set Vx = Vx SHL 1
    fn shl(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        self.v[0xF] = self.v[x] >> 7;
        self.v[x] <<= 1;
        self.pc += 2;
    }

    // 9xy0 : Skip next instruction if Vx != Vy
    fn sne2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        self.pc += if self.v[x] != self.v[y] { 4 } else { 2 };
    }

    // Annn : Set I = nnn
    fn ld03(&mut self, opcode: u16) {
        self.i = opcode & 0x0FFF;
        self.pc += 2;
    }

    // Bnnn : Jump to location nnn + V0
    fn jp2(&mut self, opcode: u16) {
        self.pc = (opcode & 0x0FFF) + (self.v[0] as u16);
    }

    // Cxkk : Set Vx = random byte AND kk
    fn rnd(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let rand: u8 = rand::prelude::random();

        self.v[x] = rand & kk;
        self.pc += 2;
    }

    // Dxyn : Display n-byte sprite starting at memory location I
    // at (Vx, Vy), set VF = collision.
    fn drw(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as usize;

        self.v[0xF] = 0;
        for dy in 0..n {
            let sprite = self.memory[(self.i as usize) + dy];
            for dx in 0..8 {
                if sprite & (0x80 >> dx) != 0 {
                    // Modulo display width and height to wrap parts of
                    // sprite that fall outside the display coordinates
                    let xpos = ((self.v[x] as usize) + dx) % DISPLAY_W;
                    let ypos = ((self.v[y] as usize) + dy) % DISPLAY_H;

                    let displayed = self.display[ypos * DISPLAY_W + xpos];
                    if displayed {
                        self.v[0xF] = 1;
                    }
                    self.display[ypos * DISPLAY_W + xpos] = !displayed;
                }
            }
        }
        self.pc += 2;
    }

    // Ex9E : Skip next instruction if key with the value of Vx is pressed
    fn skp(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let val = self.v[x] as usize;

        self.pc += if let Some(true) = self.keyboard.get(val) {
            4
        } else {
            2
        };
    }

    // ExA1 : Skip next instruction if key with the value of Vx is not pressed
    fn sknp(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let val = self.v[x] as usize;

        self.pc += if let Some(false) = self.keyboard.get(val) {
            4
        } else {
            2
        };
    }

    // Fx07 : Set Vx = delay timer value
    fn ld04(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        self.v[x] = self.delay_timer;
        self.pc += 2;
    }

    // Fx0A : Wait for a key press, store the value of the key in Vx
    fn ld05(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        let mut key_pressed = false;
        for (i, &key) in self.keyboard.iter().enumerate() {
            if key {
                self.v[x] = i as u8;
                key_pressed = true;
            }
        }

        // Skip cycle. All execution stops until a key is pressed
        if !key_pressed {
            return;
        } else {
            self.pc += 2;
        }
    }

    // Fx15 : Set delay timer = Vx
    fn ld06(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        self.delay_timer = self.v[x];
        self.pc += 2;
    }

    // Fx18 : Set sound timer = Vx
    fn ld07(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        self.sound_timer = self.v[x];
        self.pc += 2;
    }

    // Fx1E : Set I = I + Vx
    fn add3(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        self.i += self.v[x] as u16;
        self.pc += 2;
    }

    // Fx29 : Set I = location of sprite for digit Vx
    fn ld08(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        self.i = (self.v[x] as u16) * 0x5; // Sprites are 5 bytes long
        self.pc += 2;
    }

    // Fx33 : Store BCD representation of Vx in memory locations
    // I, I+1, and I+2
    fn ld09(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let i = self.i as usize;
        let val = self.v[x];

        self.memory[i] = val / 100;
        self.memory[i + 1] = (val / 10) % 10;
        self.memory[i + 2] = val % 10;
        self.pc += 2;
    }

    // Fx55 : Store registers V0 through Vx in memory starting at location I
    fn ld10(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        for i in 0..=x {
            let index = (self.i as usize) + i;
            self.memory[index] = self.v[i];
        }
        self.pc += 2;
    }

    // Fx65 : Read registers V0 through Vx from memory starting at location I
    fn ld11(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        for i in 0..=x {
            let index = (self.i as usize) + i;
            self.v[i] = self.memory[index];
        }
        self.pc += 2;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_init() {
        let c = Chip8::init();

        assert_eq!(c.memory.len(), 4096);
        assert_eq!(c.memory[..80], HEX_SPRITES[..]);
        assert!(c.memory[80..].iter().all(|&x| x == 0));

        assert_eq!(c.v.len(), 16);
        assert!(c.v.iter().all(|&x| x == 0));

        assert_eq!(c.pc, 0x200);

        assert_eq!(c.stack.len(), 16);
        assert!(c.stack.iter().all(|&x| x == 0));

        assert_eq!(c.keyboard.len(), 16);
        assert!(c.keyboard.iter().all(|&x| x == false));

        assert_eq!(c.display.len(), 64 * 32);
    }

    #[test]
    fn test_cls() {
        let mut c = Chip8::init();

        c.display[0] = true;
        c.display[31 * DISPLAY_W + 63] = true;

        c.execute_opcode_internal(0x00E0);

        assert!(c.display.iter().all(|&x| x == false));
    }

    #[test]
    fn test_ret() {
        let mut c = Chip8::init();

        c.sp = 5;
        c.stack[(c.sp - 1) as usize] = 0xEEE;

        c.execute_opcode_internal(0x00EE);

        assert_eq!(c.sp, 4);
        assert_eq!(c.pc, 0xEEE + 2);
    }

    #[test]
    fn test_jp1() {
        let mut c = Chip8::init();

        c.execute_opcode_internal(0x1ABC);

        assert_eq!(c.pc, 0xABC);
    }

    #[test]
    fn test_call() {
        let mut c = Chip8::init();

        c.execute_opcode_internal(0x2ABC);

        assert_eq!(c.sp, 1);
        assert_eq!(c.stack[0], 0x200);
        assert_eq!(c.pc, 0xABC);
    }

    #[test]
    fn test_se1_skip() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xBC;

        c.execute_opcode_internal(0x3ABC);

        assert_eq!(c.pc, 0x200 + 4);
    }

    #[test]
    fn test_se1_noskip() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xBD;

        c.execute_opcode_internal(0x3ABC);

        assert_eq!(c.pc, 0x200 + 2);
    }

    #[test]
    fn test_sne1_skip() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xBD;

        c.execute_opcode_internal(0x4ABC);

        assert_eq!(c.pc, 0x200 + 4);
    }

    #[test]
    fn test_sne1_noskip() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xBC;

        c.execute_opcode_internal(0x4ABC);

        assert_eq!(c.pc, 0x200 + 2);
    }

    #[test]
    fn test_se2_skip() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xCD;
        c.v[0xB] = 0xCD;

        c.execute_opcode_internal(0x5AB0);

        assert_eq!(c.pc, 0x200 + 4);
    }

    #[test]
    fn test_se2_noskip() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xCD;
        c.v[0xB] = 0xCE;

        c.execute_opcode_internal(0x5AB0);

        assert_eq!(c.pc, 0x200 + 2);
    }

    #[test]
    fn test_ld01() {
        let mut c = Chip8::init();

        c.execute_opcode_internal(0x6ABC);

        assert_eq!(c.v[0xA], 0xBC);
    }

    #[test]
    fn test_add1() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x10;

        c.execute_opcode_internal(0x7ABC);

        assert_eq!(c.v[0xA], 0x10 + 0xBC);
    }

    #[test]
    fn test_ld02() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xCD;
        c.v[0xB] = 0xEF;

        c.execute_opcode_internal(0x8AB0);

        assert_eq!(c.v[0xA], 0xEF);
    }

    #[test]
    fn test_or() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xC0;
        c.v[0xB] = 0x0D;

        c.execute_opcode_internal(0x8AB1);

        assert_eq!(c.v[0xA], 0xCD);
    }

    #[test]
    fn test_and() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xCD;
        c.v[0xB] = 0xCE;

        c.execute_opcode_internal(0x8AB2);

        assert_eq!(c.v[0xA], 0xCC);
    }

    #[test]
    fn test_xor() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xCD;
        c.v[0xB] = 0xCE;

        c.execute_opcode_internal(0x8AB3);

        assert_eq!(c.v[0xA], 0x03);
    }

    #[test]
    fn test_add2_nocarry() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x11;
        c.v[0xB] = 0x12;

        c.execute_opcode_internal(0x8AB4);

        assert_eq!(c.v[0xA], 0x23);
        assert_eq!(c.v[0xF], 0);
    }

    #[test]
    fn test_add2_carry() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xFF;
        c.v[0xB] = 0xFF;

        c.execute_opcode_internal(0x8AB4);

        assert_eq!(c.v[0xA], 0xFE);
        assert_eq!(c.v[0xF], 1);
    }

    #[test]
    fn test_sub_noborrow() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xFF;
        c.v[0xB] = 0xFE;

        c.execute_opcode_internal(0x8AB5);

        assert_eq!(c.v[0xA], 0x01);
        assert_eq!(c.v[0xF], 1);
    }

    #[test]
    fn test_sub_borrow() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x11;
        c.v[0xB] = 0x12;

        c.execute_opcode_internal(0x8AB5);

        assert_eq!(c.v[0xA], 0xFF);
        assert_eq!(c.v[0xF], 0);
    }

    #[test]
    fn test_shr_nolsb() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x22;

        c.execute_opcode_internal(0x8AB6);

        assert_eq!(c.v[0xA], 0x11);
        assert_eq!(c.v[0xF], 0);
    }

    #[test]
    fn test_shr_lsb() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x21;

        c.execute_opcode_internal(0x8AB6);

        assert_eq!(c.v[0xA], 0x10);
        assert_eq!(c.v[0xF], 1);
    }

    #[test]
    fn test_subn_borrow() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xFF;
        c.v[0xB] = 0xFE;

        c.execute_opcode_internal(0x8AB7);

        assert_eq!(c.v[0xA], 0xFF);
        assert_eq!(c.v[0xF], 0);
    }

    #[test]
    fn test_subn_noborrow() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x11;
        c.v[0xB] = 0x12;

        c.execute_opcode_internal(0x8AB7);

        assert_eq!(c.v[0xA], 0x01);
        assert_eq!(c.v[0xF], 1);
    }

    #[test]
    fn test_shr_nomsb() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x22;

        c.execute_opcode_internal(0x8ABE);

        assert_eq!(c.v[0xA], 0x44);
        assert_eq!(c.v[0xF], 0);
    }

    #[test]
    fn test_shr_msb() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xC0;

        c.execute_opcode_internal(0x8ABE);

        assert_eq!(c.v[0xA], 0x80);
        assert_eq!(c.v[0xF], 1);
    }

    #[test]
    fn test_sne2_noskip() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xCD;
        c.v[0xB] = 0xCD;

        c.execute_opcode_internal(0x9AB0);

        assert_eq!(c.pc, 0x200 + 2);
    }

    #[test]
    fn test_sne2_skip() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xCD;
        c.v[0xB] = 0xCE;

        c.execute_opcode_internal(0x9AB0);

        assert_eq!(c.pc, 0x200 + 4);
    }

    #[test]
    fn test_ld03() {
        let mut c = Chip8::init();

        c.execute_opcode_internal(0xA123);

        assert_eq!(c.i, 0x123);
    }

    #[test]
    fn test_jp2() {
        let mut c = Chip8::init();

        c.v[0] = 0x55;

        c.execute_opcode_internal(0xB123);

        assert_eq!(c.pc, 0x178);
    }

    #[test]
    fn test_add3() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x10;
        c.i = 0xAB0C;

        c.execute_opcode_internal(0xFA1E);

        assert_eq!(c.i, 0xAB1C);
    }

    #[test]
    fn test_drw_nowrap() {
        let mut c = Chip8::init();

        c.i = 0x500;
        c.memory[0x500] = 0b00011000;
        c.memory[0x501] = 0b00100100;
        c.memory[0x502] = 0b01000010;
        c.memory[0x503] = 0b10000001;
        c.v[0xA] = 0x05;
        c.v[0xB] = 0x0A;

        c.execute_opcode_internal(0xDAB4);

        let i = 0x0A * DISPLAY_W;
        assert!(c.display[..i].iter().all(|&x| x == false));

        let i = (0x0A + 4) * DISPLAY_W;
        assert!(c.display[i..].iter().all(|&x| x == false));

        let i = 0x0A * DISPLAY_W + 0x05;
        assert_eq!(
            &c.display[i..(i + 8)],
            &[false, false, false, true, true, false, false, false]
        );

        let i = 0x0B * DISPLAY_W + 0x05;
        assert_eq!(
            &c.display[i..(i + 8)],
            &[false, false, true, false, false, true, false, false]
        );

        let i = 0x0C * DISPLAY_W + 0x05;
        assert_eq!(
            &c.display[i..(i + 8)],
            &[false, true, false, false, false, false, true, false]
        );

        let i = 0x0D * DISPLAY_W + 0x05;
        assert_eq!(
            &c.display[i..(i + 8)],
            &[true, false, false, false, false, false, false, true]
        );

        assert_eq!(c.v[0xF], 0);
    }

    #[test]
    fn test_drw_wrapx() {
        let mut c = Chip8::init();

        c.i = 0x500;
        c.memory[0x500] = 0b10101011;
        c.v[0xA] = 60;
        c.v[0xB] = 0;

        c.execute_opcode_internal(0xDAB1);

        assert_eq!(&c.display[60..DISPLAY_W], &[true, false, true, false]);
        assert_eq!(&c.display[..4], &[true, false, true, true]);
        assert_eq!(c.v[0xF], 0);
    }

    #[test]
    fn test_drw_wrapy() {
        let mut c = Chip8::init();

        c.i = 0x500;
        c.memory[0x500] = 0b10000000;
        c.memory[0x501] = 0b01000000;
        c.memory[0x502] = 0b00100000;
        c.memory[0x503] = 0b00010000;
        c.v[0xA] = 0;
        c.v[0xB] = 30;

        c.execute_opcode_internal(0xDAB4);

        let i = 30 * DISPLAY_W;
        assert_eq!(&c.display[i..(i + 4)], &[true, false, false, false]);

        let i = 31 * DISPLAY_W;
        assert_eq!(&c.display[i..(i + 4)], &[false, true, false, false]);

        let i = 0 * DISPLAY_W;
        assert_eq!(&c.display[i..(i + 4)], &[false, false, true, false]);

        let i = 1 * DISPLAY_W;
        assert_eq!(&c.display[i..(i + 4)], &[false, false, false, true]);

        assert_eq!(c.v[0xF], 0);
    }

    #[test]
    fn test_drw_collision() {
        let mut c = Chip8::init();

        c.i = 0x500;
        c.memory[0x500] = 0b11000000;
        c.v[0xA] = 0;
        c.v[0xB] = 0;
        c.display[0] = true;

        c.execute_opcode_internal(0xDAB1);

        assert_eq!(c.display[0], false);
        assert_eq!(c.display[1], true);
        assert_eq!(c.v[0xF], 1);
    }

    #[test]
    fn test_skp_press() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x05;
        c.keyboard[0x05] = true;

        c.execute_opcode_internal(0xEA9E);

        assert_eq!(c.pc, 0x200 + 4);
    }

    #[test]
    fn test_skp_nopress() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x05;
        c.keyboard[0x05] = false;

        c.execute_opcode_internal(0xEA9E);

        assert_eq!(c.pc, 0x200 + 2);
    }

    #[test]
    fn test_sknp_press() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x05;
        c.keyboard[0x05] = true;

        c.execute_opcode_internal(0xEAA1);

        assert_eq!(c.pc, 0x200 + 2);
    }

    #[test]
    fn test_sknp_nopress() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x05;
        c.keyboard[0x05] = false;

        c.execute_opcode_internal(0xEAA1);

        assert_eq!(c.pc, 0x200 + 4);
    }

    #[test]
    fn test_ld04() {
        let mut c = Chip8::init();

        c.delay_timer = 0xAB;
        c.execute_opcode_internal(0xFA07);

        assert_eq!(c.v[0xA], 0xAB);
    }

    #[test]
    fn test_ld05_press() {
        let mut c = Chip8::init();

        c.keyboard[0x05] = true;

        c.execute_opcode_internal(0xFA0A);

        assert_eq!(c.v[0xA], 0x05);
        assert_eq!(c.pc, 0x200 + 2);
    }

    #[test]
    fn test_ld05_nopress() {
        let mut c = Chip8::init();

        c.execute_opcode_internal(0xFA0A);

        assert_eq!(c.v[0xA], 0x0);
        assert_eq!(c.pc, 0x200);
    }

    #[test]
    fn test_ld06() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xCD;
        c.execute_opcode_internal(0xFA15);

        assert_eq!(c.delay_timer, 0xCD - 1);
    }

    #[test]
    fn test_ld07() {
        let mut c = Chip8::init();

        c.v[0xA] = 0xCD;
        c.execute_opcode_internal(0xFA18);

        assert_eq!(c.sound_timer, 0xCD - 1);
    }

    #[test]
    fn test_ld08() {
        let mut c = Chip8::init();

        c.v[0xA] = 0x2;
        c.execute_opcode_internal(0xFA29);

        assert_eq!(c.i, 0xA);
        assert_eq!(c.memory[(c.i as usize)], 0xF0);
        assert_eq!(c.memory[(c.i as usize) + 1], 0x10);
        assert_eq!(c.memory[(c.i as usize) + 2], 0xF0);
        assert_eq!(c.memory[(c.i as usize) + 3], 0x80);
        assert_eq!(c.memory[(c.i as usize) + 4], 0xF0);
    }

    #[test]
    fn test_ld09() {
        let mut c = Chip8::init();

        c.i = 0x500;
        c.v[0xA] = 234;
        c.execute_opcode_internal(0xFA33);

        assert_eq!(c.memory[0x500], 0x2);
        assert_eq!(c.memory[0x501], 0x3);
        assert_eq!(c.memory[0x502], 0x4);
        assert_eq!(c.memory[0x503], 0x0);
    }

    #[test]
    fn test_ld10() {
        let mut c = Chip8::init();

        c.i = 0x500;
        c.v[0x0] = 0x1;
        c.v[0x1] = 0xA;
        c.v[0x2] = 0xF;
        c.execute_opcode_internal(0xF255);

        assert_eq!(c.memory[0x500], 0x1);
        assert_eq!(c.memory[0x501], 0xA);
        assert_eq!(c.memory[0x502], 0xF);
        assert_eq!(c.memory[0x503], 0x0);
    }

    #[test]
    fn test_ld11() {
        let mut c = Chip8::init();

        c.i = 0x500;
        c.memory[0x500] = 0x1;
        c.memory[0x501] = 0xA;
        c.memory[0x502] = 0xF;
        c.execute_opcode_internal(0xF265);

        assert_eq!(c.v[0x0], 0x1);
        assert_eq!(c.v[0x1], 0xA);
        assert_eq!(c.v[0x2], 0xF);
        assert_eq!(c.v[0x3], 0x0);
    }
}
