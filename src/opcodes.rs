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
            _ => panic!("Byte doesn't map to any opcode."),
        }
    }
}
