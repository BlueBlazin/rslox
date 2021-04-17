use crate::error::{LoxError, Result};
use crate::value::ValueHandle;

#[derive(Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Vec<ValueHandle>,
}

impl Chunk {
    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, handle: ValueHandle) -> Result<u8> {
        if self.constants.len() >= 256 {
            return Err(LoxError::CompileError);
        }
        self.constants.push(handle);
        Ok(self.constants.len() as u8 - 1)
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            code: Vec::with_capacity(8),
            lines: Vec::with_capacity(8),
            constants: Vec::with_capacity(4),
        }
    }
}
