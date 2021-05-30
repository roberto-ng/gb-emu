pub enum Instruction {
    ADD { target: ArithmeticTarget, },
}

pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

