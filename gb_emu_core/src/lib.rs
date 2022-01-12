pub mod cpu;
pub mod cpu_registers;
pub mod instruction;

#[cfg(test)]
mod tests {
    use crate::cpu::Cpu;

    #[test]
    fn it_works() {
        let _cpu = Cpu::new();
        assert_eq!(2 + 2, 4);
    }
}
