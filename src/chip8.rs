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
