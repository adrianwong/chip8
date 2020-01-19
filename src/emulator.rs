use crate::chip8::{Chip8, DISPLAY_H, DISPLAY_W};
use minifb::{Key, Scale, Window, WindowOptions};
use std::io;

pub fn run_emulator(fname: &str) -> Result<(), io::Error> {
    let mut chip8 = Chip8::load_rom(&fname)?;

    let mut buf = vec![0u32; DISPLAY_W * DISPLAY_H];

    let mut window = Window::new(
        "Baby's First (CHIP-8) Emulator (ESC to exit)",
        DISPLAY_W,
        DISPLAY_H,
        WindowOptions {
            scale: Scale::X16,
            ..WindowOptions::default()
        },
    )
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    while window.is_open() {
        chip8.execute_opcode();

        for (i, b) in buf.iter_mut().enumerate() {
            *b = if chip8.display()[i] { 0xFFFFFF } else { 0 };
        }

        if let Some(keys) = window.get_keys() {
            match keys.first() {
                Some(Key::Escape) => break,
                Some(key) => {
                    if let Some(chip8_key) = to_chip8_key(*key) {
                        chip8.set_key(chip8_key);
                    }
                }
                None => chip8.reset_keys(),
            }
        }

        window
            .update_with_buffer(&buf, DISPLAY_W, DISPLAY_H)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    Ok(())
}

fn to_chip8_key(key: Key) -> Option<u8> {
    match key {
        Key::Key1 => Some(0x1),
        Key::Key2 => Some(0x2),
        Key::Key3 => Some(0x3),
        Key::Key4 => Some(0xC),
        Key::Q => Some(0x4),
        Key::W => Some(0x5),
        Key::E => Some(0x6),
        Key::R => Some(0xD),
        Key::A => Some(0x7),
        Key::S => Some(0x8),
        Key::D => Some(0x9),
        Key::F => Some(0xE),
        Key::Z => Some(0xA),
        Key::X => Some(0x0),
        Key::C => Some(0xB),
        Key::V => Some(0xF),
        _ => None,
    }
}
