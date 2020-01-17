#[derive(Debug)]
pub struct Chip8 {
    memory: Vec<u8>,         // 4,096 bytes of RAM
    v: Vec<u8>,              // 16 general-purpose registers
    i: u16,                  // 1 I-register
    delay_timer: u8,         // Decrements at a rate of 60Hz
    sound_timer: u8,         // Decrements at a rate of 60Hz
    pc: u16,                 // Program counter
    sp: u8,                  // Stack pointer
    stack: Vec<u16>,         // 16 stack levels
    keyboard: Vec<bool>,     // 16-key hexadecimal keypad
    display: Vec<Vec<bool>>, // 64 x 32 monochrome display
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

impl Chip8 {
    pub fn init() -> Chip8 {
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
            display: vec![vec![false; 64]; 32],
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
                0x00E0 => todo!(),
                0x00EE => todo!(),
                _ => panic!("Unknown opcode: {:X?}", opcode),
            },
            0x1000 => todo!(),
            0x2000 => todo!(),
            0x3000 => todo!(),
            0x4000 => todo!(),
            0x5000 => todo!(),
            0x6000 => todo!(),
            0x7000 => todo!(),
            0x8000 => match opcode & 0x000F {
                0x0000 => todo!(),
                0x0001 => todo!(),
                0x0002 => todo!(),
                0x0003 => todo!(),
                0x0004 => todo!(),
                0x0005 => todo!(),
                0x0006 => todo!(),
                0x0007 => todo!(),
                0x000E => todo!(),
                _ => panic!("Unknown opcode: {:X?}", opcode),
            },
            0x9000 => todo!(),
            0xA000 => todo!(),
            0xB000 => todo!(),
            0xC000 => todo!(),
            0xD000 => todo!(),
            0xE000 => match opcode & 0x00FF {
                0x009E => todo!(),
                0x00A1 => todo!(),
                _ => panic!("Unknown opcode: {:X?}", opcode),
            },
            0xF000 => match opcode & 0x00FF {
                0x0007 => todo!(),
                0x000A => todo!(),
                0x0015 => todo!(),
                0x0018 => todo!(),
                0x001E => todo!(),
                0x0029 => todo!(),
                0x0033 => todo!(),
                0x0055 => todo!(),
                0x0065 => todo!(),
                _ => panic!("Unknown opcode: {:X?}", opcode),
            },
            _ => panic!("Unknown opcode: {:X?}", opcode),
        }
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

        assert_eq!(c.display.len(), 32);
        for row in &c.display {
            assert_eq!(row.len(), 64);
            assert!(row.iter().all(|&x| x == false));
        }
    }
}
