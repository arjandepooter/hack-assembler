use std::default::Default;

#[derive(Debug, PartialEq, Clone)]
pub enum AValue {
    Label(String),
    Value(u16),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    AInstruction(self::AValue),
    CInstruction {
        instruction: u8,
        destination: u8,
        jump: u8,
    },
    Label(String),
    Noop,
}

impl Default for Instruction {
    fn default() -> Self {
        Instruction::Noop
    }
}
