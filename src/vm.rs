use crate::chunk::Chunk;
use crate::error::{LoxError, Result};
use crate::opcodes::OpCode;
use crate::value::Value;

const STACK_MAX: usize = 256;

macro_rules! binary_op {
    ($op:tt, $self:expr) => {{
        let b = $self.pop_number()?;
        let a = $self.pop_number()?;
        $self.push(Value::Number(a $op b))?;
    }};

    ($op:tt, $self:expr, $type:tt) => {{
        let b = $self.pop_number()?;
        let a = $self.pop_number()?;
        $self.push(Value::$type(a $op b))?;
    }};
}

macro_rules! push_value {
    ($value:expr, $self:expr) => {{
        $self.push($value)?;
    }};
}

pub struct Vm {
    pub stack: Vec<Value>,
    chunk: Chunk,
    ip: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(STACK_MAX),
            chunk: Chunk::new(String::from("")),
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
                    push_value!(Value::Number(value), self);
                }
                OpCode::Negate => {
                    let value = self.pop_number()?;
                    push_value!(Value::Number(-value), self);
                }

                OpCode::Add => binary_op!(+, self),
                OpCode::Subtract => binary_op!(-, self),
                OpCode::Multiply => binary_op!(*, self),
                OpCode::Divide => binary_op!(/, self),

                OpCode::Nil => push_value!(Value::Nil, self),
                OpCode::True => push_value!(Value::Bool(true), self),
                OpCode::False => push_value!(Value::Bool(false), self),

                OpCode::Not => {
                    let value = self.pop()?.is_falsey();
                    push_value!(Value::Bool(value), self);
                }
                OpCode::Equal => binary_op!(==, self, Bool),
                OpCode::Greater => binary_op!(>, self, Bool),
                OpCode::Less => binary_op!(<, self, Bool),
            }
        }
    }

    #[inline]
    fn fetch(&mut self) -> u8 {
        self.ip += 1;

        self.chunk.code[self.ip - 1]
    }

    #[inline]
    fn fetch_const(&mut self) -> f64 {
        let idx = self.fetch() as usize;

        self.chunk.constants[idx]
    }

    #[inline]
    fn push(&mut self, value: Value) -> Result<()> {
        if self.stack.len() < STACK_MAX {
            self.stack.push(value);

            Ok(())
        } else {
            Err(LoxError::StackOverflow)
        }
    }

    #[inline]
    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(LoxError::StackUnderflow)
    }

    fn pop_number(&mut self) -> Result<f64> {
        match self.pop()? {
            Value::Number(n) => Ok(n),
            _ => Err(LoxError::TypeError),
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
        // assert_eq!(&Value::Number(3.0), &vm.stack[0]);
    }
}
