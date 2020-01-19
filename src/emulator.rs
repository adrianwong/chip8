use crate::chip8::{Chip8, DISPLAY_H, DISPLAY_W};
use minifb::{Scale, Window, WindowOptions};
use std::io;

pub fn run_emulator(fname: &str) -> Result<(), io::Error> {
    let mut chip8 = Chip8::load_rom(&fname)?;

    let mut buf = vec![0u32; DISPLAY_W * DISPLAY_H];

    let mut window = Window::new(
        "Baby's First (CHIP-8) Emulator",
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

        window
            .update_with_buffer(&buf, DISPLAY_W, DISPLAY_H)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    }

    Ok(())
}
