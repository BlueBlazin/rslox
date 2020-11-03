use crate::error::{LoxError, Result};

#[derive(Debug, PartialEq, Clone)]
pub enum Const {
    Num(f64),
    Str(String),
}

pub struct Chunk {
    pub name: String,
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<Const>,
}

impl Chunk {
    pub fn new(name: String) -> Self {
        Self {
            name,
            code: Vec::with_capacity(8),
            lines: Vec::with_capacity(8),
            constants: Vec::with_capacity(4),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Const) -> Result<u8> {
        if self.constants.len() >= 256 {
            return Err(LoxError::CompileError);
        }
        self.constants.push(value);
        Ok(self.constants.len() as u8 - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcodes::OpCode;

    #[test]
    fn test_create_chunk() {
        let mut chunk = Chunk::new(String::from("Test"));
        chunk.add_constant(Const::Num(7.0));
        chunk.add_constant(Const::Num(42.0));
        chunk.write(OpCode::Add as u8, 0);
        chunk.write(OpCode::Constant as u8, 0);
        chunk.write(1, 0);
        chunk.write(OpCode::Constant as u8, 0);
        chunk.write(0, 0);
        chunk.write(OpCode::Return as u8, 2);
        println!("{:?}", chunk);
    }
}
