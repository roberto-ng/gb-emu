pub mod cartridge;
pub mod cpu;
pub mod cpu_registers;
pub mod error;
pub mod gpu;
pub mod instruction;
pub mod interrupt;
pub mod memory_bus;
pub mod timer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
