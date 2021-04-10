#[derive(Debug, PartialEq)]
pub enum OpCode {
    Return,
    Constant,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Nil,
    True,
    False,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    Jump,
    Loop,
    Call,
    Closure,
    GetUpvalue,
    SetUpvalue,
}

impl From<u8> for OpCode {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => OpCode::Return,
            0x01 => OpCode::Constant,
            0x02 => OpCode::Negate,
            0x03 => OpCode::Add,
            0x04 => OpCode::Subtract,
            0x05 => OpCode::Multiply,
            0x06 => OpCode::Divide,
            0x07 => OpCode::Nil,
            0x08 => OpCode::True,
            0x09 => OpCode::False,
            0x0A => OpCode::Not,
            0x0B => OpCode::Equal,
            0x0C => OpCode::Greater,
            0x0D => OpCode::Less,
            0x0E => OpCode::Print,
            0x0F => OpCode::Pop,
            0x10 => OpCode::DefineGlobal,
            0x11 => OpCode::GetGlobal,
            0x12 => OpCode::SetGlobal,
            0x13 => OpCode::GetLocal,
            0x14 => OpCode::SetLocal,
            0x15 => OpCode::JumpIfFalse,
            0x16 => OpCode::Jump,
            0x17 => OpCode::Loop,
            0x18 => OpCode::Call,
            0x19 => OpCode::Closure,
            0x1A => OpCode::GetUpvalue,
            0x1B => OpCode::SetUpvalue,
            _ => panic!("Byte doesn't map to any opcode."),
        }
    }
}
