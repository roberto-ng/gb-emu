use std::env;
use std::fs;

use gb_emu_common::GameBoy;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("No args");
        return Ok(());
    }

    let rom_path = &args[1];
    let rom = fs::read(rom_path)?;
    let mut gb = GameBoy::new();
    gb.load_rom(rom)?;
    println!("ROM loaded successfully {:#06X}", gb.cpu.pc);

    let mut i = 1;
    loop {
        let pc = gb.cpu.pc;
        let mut bytes = vec![];
        for i in 0..5 {
            let byte = gb.cpu.bus.read_byte(pc + i)?;
            bytes.push(byte);
        }
        
        println!(
            "{i} - {pc:04X} - {bytes:02X?} - A = {a:02X}, Z = {z}", 
            pc = gb.cpu.pc, 
            a = gb.cpu.registers.a,
            z = gb.cpu.registers.f.zero,
        );

        let result = gb.step();
        if let Err(err) = result {
            println!("{err}");
            std::process::exit(1);
        }

        i += 1;
    }
}
