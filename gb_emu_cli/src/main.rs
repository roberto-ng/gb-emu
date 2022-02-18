use std::fs;
use clap::Parser;
use gb_emu_common::GameBoy;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    input_file: String,

    #[clap(long)]
    log: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let rom_path = args.input_file;
    let rom = fs::read(rom_path)?;
    let mut gb = GameBoy::new();
    gb.load_rom(rom)?;

    let mut i = 1;
    loop {
        let pc = gb.cpu.pc;
        let mut bytes = vec![];
        for i in 0..5 {
            let byte = gb.cpu.bus.read_byte(pc + i)?;
            bytes.push(byte);
        }
        
        if args.log {
            let pc = gb.cpu.pc;
            let a = gb.cpu.registers.a;
            let f: u8 = gb.cpu.registers.f.into();
            let h = gb.cpu.registers.h;
            let l = gb.cpu.registers.l;
            let sp = gb.cpu.sp;
            println!("{i} - pc = {pc:04X} - {bytes:02X?} - A = {a:02X}, F = {f:02X} H = {h:02X}, L = {l:02X}, SP = {sp:04X}");
        }

        let result = gb.step();
        if let Err(err) = result {
            println!("{err}");
            std::process::exit(1);
        }

        i += 1;
    }
}
