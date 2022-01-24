use std::env;
use std::fs;

use gb_emu_common::cartridge::header::Header;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return;
    }

    let rom_path = &args[1];
    let rom = fs::read(rom_path)
        .expect("Could not read file");
    let header = Header::read_rom_header(&rom)
        .expect("Error while reading ROM header");
    let rom_title = header.title
        .unwrap_or(String::from("NO TITLE"));
    let cartridge_type = header.cartridge_type;

    println!("Title: {rom_title}");
    println!("Cartridge type: {cartridge_type}");
}
