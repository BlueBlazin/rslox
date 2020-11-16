use crate::chunk::Const;
use crate::error::Result;

pub trait Codegen {
    fn emit_byte(&mut self, value: u8);
    fn emit_const(&mut self, value: Const) -> Result<()>;
    fn emit_jump(&mut self, value: u8) -> usize;
    fn emit_loop(&mut self, loop_start: usize) -> Result<()>;

    fn emit_bytes(&mut self, value1: u8, value2: u8) {
        self.emit_byte(value1);
        self.emit_byte(value2);
    }
}
