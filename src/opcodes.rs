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
            _ => panic!("Byte doesn't map to any opcode."),
        }
    }
}
