use crate::value::Value;

pub trait Codegen {
    fn emit_byte(&mut self, value: u8);
    fn emit_const(&mut self, value: Value);

    fn emit_bytes(&mut self, value1: u8, value2: u8) {
        self.emit_byte(value1);
        self.emit_byte(value2);
    }
}
