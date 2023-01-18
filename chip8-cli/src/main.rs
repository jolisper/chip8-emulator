use std::io::Read;
use chip8_avsys;

use clap::Parser;

#[derive(Parser)]
struct Args {
    rom_file: String,
    debug: bool,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let rom_file_name = args.rom_file;
    let debug_mode = args.debug;

    // Load ROM file
    let mut file = std::fs::File::open(rom_file_name).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).expect("read all ROM file");

    chip8_avsys::start(buf, debug_mode)
}
