use crate::opcodes::OpCode;
use crate::value::Value;
use std::fmt;

pub struct Chunk {
    pub name: String,
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn new(name: String) -> Self {
        Self {
            name,
            code: Vec::with_capacity(8),
            lines: Vec::with_capacity(8),
            constants: Vec::with_capacity(2),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        // TODO: There is a bug here when constants.len >= 256
        self.constants.len() as u8 - 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_chunk() {
        let mut chunk = Chunk::new(String::from("Test"));
        chunk.constants.push(7.0);
        chunk.constants.push(42.0);
        chunk.write(OpCode::Add as u8, 0);
        chunk.write(OpCode::Constant as u8, 0);
        chunk.write(1, 0);
        chunk.write(OpCode::Constant as u8, 0);
        chunk.write(0, 0);
        chunk.write(OpCode::Return as u8, 2);
        println!("{:?}", chunk);
    }
}
