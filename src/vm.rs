use crate::chunk::{Chunk, Const};
use crate::error::{LoxError, Result};
use crate::object::{LoxObj, ObjString};
use crate::opcodes::OpCode;
use crate::value::Value;
use broom::prelude::*;

const STACK_MAX: usize = 256;

macro_rules! binary_op {
    ($op:tt, $self:expr) => {{
        let b = $self.pop_number()?;
        let a = $self.pop_number()?;
        push_value!(Value::Number(a $op b), $self);
    }};

    ($op:tt, $self:expr, $type:tt) => {{
        let b = $self.pop_number()?;
        let a = $self.pop_number()?;
        push_value!(Value::$type(a $op b), $self);
    }};
}

macro_rules! push_value {
    ($value:expr, $self:expr) => {{
        let handle = $self.alloc($value);
        $self.push(handle)?;
    }};
}

pub struct Vm {
    pub stack: Vec<Rooted<Value>>,
    pub heap: Heap<Value>,
    chunk: Chunk,
    ip: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(STACK_MAX),
            heap: Heap::default(),
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
                OpCode::Constant => match self.fetch_const() {
                    Const::Num(n) => push_value!(Value::Number(n), self),
                    Const::Str(n) => unimplemented!(),
                },
                OpCode::Negate => {
                    let value = self.pop_number()?;
                    push_value!(Value::Number(-value), self);
                }
                OpCode::Add => binary_op!(-, self),
                OpCode::Subtract => binary_op!(-, self),
                OpCode::Multiply => binary_op!(*, self),
                OpCode::Divide => binary_op!(/, self),

                OpCode::Nil => push_value!(Value::Nil, self),
                OpCode::True => push_value!(Value::Bool(true), self),
                OpCode::False => push_value!(Value::Bool(false), self),

                OpCode::Not => {
                    let handle = self.pop()?;

                    let value = self
                        .heap
                        .get(handle)
                        .ok_or(LoxError::RuntimeError)?
                        .is_falsey();

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
    fn fetch_const(&mut self) -> Const {
        let idx = self.fetch() as usize;

        self.chunk.constants[idx].clone()
    }

    #[inline]
    fn push(&mut self, value: Rooted<Value>) -> Result<()> {
        if self.stack.len() < STACK_MAX {
            self.stack.push(value);

            Ok(())
        } else {
            Err(LoxError::StackOverflow)
        }
    }

    #[inline]
    fn pop(&mut self) -> Result<Rooted<Value>> {
        self.stack.pop().ok_or(LoxError::StackUnderflow)
    }

    fn pop_number(&mut self) -> Result<f64> {
        let handle = self.pop()?;

        match self.heap.get(handle) {
            Some(Value::Number(n)) => Ok(*n),
            _ => Err(LoxError::TypeError),
        }
    }

    fn alloc(&mut self, value: Value) -> Rooted<Value> {
        self.heap.insert(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opcodes::OpCode;

    #[test]
    fn test_vm_add() {
        let mut chunk = Chunk::new(String::from("Test"));

        let idx = chunk.add_constant(Const::Num(1.0)).unwrap();
        chunk.write(OpCode::Constant as u8, 0);
        chunk.write(idx, 0);

        let idx = chunk.add_constant(Const::Num(2.0)).unwrap();
        chunk.write(OpCode::Constant as u8, 0);
        chunk.write(idx, 0);

        chunk.write(OpCode::Add as u8, 0);

        let mut vm = Vm::new();

        vm.interpret(chunk).unwrap();
    }
}
