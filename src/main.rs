mod chip8;
mod emulator;

use std::env;
use std::io;

fn main() -> Result<(), io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "ROM file not specified. Usage: ./chip8 [rom_file]",
        ));
    }

    emulator::run_emulator(&args[1])?;

    Ok(())
}
