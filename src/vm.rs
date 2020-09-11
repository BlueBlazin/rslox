use crate::chunk::Chunk;
use crate::error::{LoxError, Result};
use crate::opcodes::OpCode;
use crate::value::Value;

const STACK_MAX: usize = 256;

macro_rules! binary_op {
    ($op:tt, $self:expr) => {{
        let b = $self.pop()?;
        let a = $self.pop()?;
        $self.push(a $op b)?;
        break;
    }};
}

pub struct Vm {
    chunk: Chunk,
    stack: Vec<Value>,
    sp: usize,
    ip: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(String::from("")),
            stack: vec![0.0; STACK_MAX],
            sp: 0,
            ip: 0,
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<()> {
        self.chunk = chunk;
        self.ip = 0;

        self.run()
    }

    fn run(&mut self) -> Result<()> {
        loop {
            match OpCode::from(self.fetch()) {
                OpCode::Return => {
                    println!("{:?}", self.pop()?);
                    return Ok(());
                }
                OpCode::Constant => {
                    let value = self.fetch_const();
                    self.push(value)?;
                }
                OpCode::Negate => {
                    let value = -self.pop()?;
                    self.push(value)?;
                }
                OpCode::Add => binary_op!(+, self),
                OpCode::Subtract => binary_op!(-, self),
                OpCode::Multiply => binary_op!(*, self),
                OpCode::Divide => binary_op!(/, self),
            }
        }

        Ok(())
    }

    #[inline]
    fn fetch(&mut self) -> u8 {
        self.ip += 1;

        self.chunk.code[self.ip - 1]
    }

    #[inline]
    fn fetch_const(&mut self) -> Value {
        let idx = self.fetch() as usize;

        self.chunk.constants[self.chunk.code[idx] as usize]
    }

    #[inline]
    fn push(&mut self, value: Value) -> Result<()> {
        if self.sp < STACK_MAX {
            self.stack[self.sp] = value;
            self.sp += 1;

            Ok(())
        } else {
            Err(LoxError::StackOverflow)
        }
    }

    #[inline]
    fn pop(&mut self) -> Result<Value> {
        if self.sp == 0 {
            Err(LoxError::StackUnderflow)
        } else {
            self.sp -= 1;
            Ok(self.stack[self.sp])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcodes::OpCode;

    #[test]
    fn test_vm_add() {
        let mut chunk = Chunk::new(String::from("Test"));

        let constant = chunk.add_constant(1.0);
        chunk.write(OpCode::Constant as u8, 0);
        chunk.write(constant, 0);

        let constant = chunk.add_constant(2.0);
        chunk.write(OpCode::Constant as u8, 0);
        chunk.write(constant, 0);

        chunk.write(OpCode::Add as u8, 0);

        let mut vm = Vm::new();

        vm.interpret(chunk).unwrap();
        assert_eq!(&3.0, &vm.stack[0]);
    }
}
