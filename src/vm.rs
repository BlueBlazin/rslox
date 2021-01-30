use crate::chunk::Chunk;
use crate::error::{LoxError, Result};
use crate::object::{LoxObj, ObjFunction, ObjString};
use crate::opcodes::OpCode;
use crate::value::{Value, ValueHandle};
// use broom::prelude::*;
use crate::gc::{Handle, Heap};
use std::collections::HashMap;

const STACK_MAX: usize = 256;
const FRAMES_MAX: usize = 64;

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

// macro_rules! push_value {
//     ($value:expr, $self:expr) => {{
//         let handle = $self.alloc($value);
//         $self.push(handle)?;
//     }};
// }

macro_rules! push_handle {
    ($value:expr, $self:expr) => {{
        $self.push(handle)?;
    }};
}

pub struct CallFrame {
    pub function: ObjFunction,
    pub ip: usize,
    pub slots: Vec<ValueHandle>,
}

pub struct Vm {
    pub stack: Vec<ValueHandle>,
    pub heap: Heap<Value>,
    pub frames: Vec<CallFrame>,
    globals: HashMap<String, ValueHandle>,
    // chunk: Chunk,
    function: ObjFunction,
    ip: usize,
}

impl Vm {
    pub fn new(function: ObjFunction) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            heap: Heap::new(),
            frames: Vec::with_capacity(FRAMES_MAX),
            globals: HashMap::new(),
            // chunk: Chunk::new(String::from("")),
            function,
            ip: 0,
        }
    }

    pub fn interpret(&mut self) -> Result<()> {
        // self.chunk = chunk;
        self.ip = 0;

        self.run()
    }

    fn run(&mut self) -> Result<()> {
        loop {
            match OpCode::from(self.fetch()) {
                OpCode::Return => {
                    // let handle = self.pop()?;
                    // println!("{:?}", self.heap.get(handle).ok_or(LoxError::RuntimeError)?);
                    return Ok(());
                }
                OpCode::Constant => push_value!(self.fetch_const()),
                // OpCode::Constant => match self.fetch_const() {
                //     Const::Num(n) => push_value!(Value::Number(n), self),
                //     Const::Str(s) => push_value!(
                //         Value::Obj(LoxObj::Str(Box::from(ObjString {
                //             length: s.len(),
                //             value: s,
                //         }))),
                //         self
                //     ),
                // },
                OpCode::Negate => {
                    let value = self.pop_number()?;
                    push_value!(Value::Number(-value), self);
                }
                OpCode::Add => {
                    let handle_b = self.pop()?;
                    let handle_a = self.pop()?;

                    match (self.heap.get(handle_a), self.heap.get(handle_b)) {
                        (Some(Value::Number(a)), Some(Value::Number(b))) => {
                            push_value!(Value::Number(*a + *b), self);
                        }
                        (Some(Value::Obj(LoxObj::Str(a))), Some(Value::Obj(LoxObj::Str(b)))) => {
                            let mut value = String::from(&a.value);
                            value.push_str(&b.value);

                            let new_str = Value::Obj(LoxObj::Str(Box::from(ObjString {
                                length: a.length + b.length,
                                value,
                            })));

                            push_value!(new_str, self);
                        }
                        _ => return Err(LoxError::TypeError),
                    }
                }
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
                OpCode::Equal => {
                    let handle_b = self.pop()?;
                    let handle_a = self.pop()?;

                    match (self.heap.get(handle_a), self.heap.get(handle_b)) {
                        (Some(Value::Number(a)), Some(Value::Number(b))) => {
                            push_value!(Value::Bool(a == b), self);
                        }
                        (Some(Value::Obj(LoxObj::Str(a))), Some(Value::Obj(LoxObj::Str(b)))) => {
                            push_value!(Value::Bool(a.value == b.value), self);
                        }
                        _ => return Err(LoxError::TypeError),
                    }
                }
                OpCode::Greater => binary_op!(>, self, Bool),
                OpCode::Less => binary_op!(<, self, Bool),

                OpCode::Print => {
                    let handle = self.pop()?;
                    println!(
                        "{:?}",
                        self.heap.get(handle).ok_or(LoxError::StackUnderflow)?
                    );
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::DefineGlobal => {
                    let name = self.fetch_str_const()?;
                    let handle = self.pop()?;
                    self.globals.insert(name, handle);
                }
                OpCode::GetGlobal => {
                    let name = self.fetch_str_const()?;
                    let value = self
                        .globals
                        .get(&name)
                        .ok_or(LoxError::RuntimeError)?
                        .clone();
                    self.push(value)?;
                }
                OpCode::SetGlobal => {
                    let name = self.fetch_str_const()?;
                    if !self.globals.contains_key(&name) {
                        return Err(LoxError::RuntimeError);
                    }

                    let handle = self.stack.last().ok_or(LoxError::StackUnderflow)?.clone();
                    self.globals.insert(name, handle.clone());
                }
                OpCode::GetLocal => {
                    let idx = self.fetch() as usize;
                    let handle = self.stack[idx].clone();
                    self.push(handle)?;
                }
                OpCode::SetLocal => {
                    let idx = self.fetch() as usize;
                    let handle = self.stack.last().ok_or(LoxError::StackUnderflow)?.clone();
                    self.stack[idx] = handle;
                }
                OpCode::JumpIfFalse => {
                    let offset = self.fetch16() as usize;

                    let handle = self.stack.last().ok_or(LoxError::StackUnderflow)?.clone();

                    if self.heap.get(handle).unwrap().is_falsey() {
                        self.ip += offset;
                    }
                }
                OpCode::Jump => {
                    let offset = self.fetch16() as usize;
                    self.ip += offset;
                }
                OpCode::Loop => {
                    let offset = self.fetch16() as usize;
                    self.ip -= offset;
                }
            }
        }
    }

    fn fetch_str_const(&mut self) -> Result<String> {
        let handle = self.fetch_const();

        // match self.fetch_const() {
        //     Const::Str(s) => Ok(s),
        //     _ => Err(LoxError::RuntimeError),
        // }
    }

    fn fetch16(&mut self) -> u16 {
        let hi = self.fetch();
        let lo = self.fetch();
        (hi as u16) << 8 | (lo as u16)
    }

    #[inline]
    fn fetch(&mut self) -> u8 {
        self.ip += 1;

        self.function.chunk.code[self.ip - 1]
    }

    #[inline]
    fn fetch_const(&mut self) -> ValueHandle {
        let idx = self.fetch() as usize;

        self.function.chunk.constants[idx]
    }

    #[inline]
    fn push(&mut self, handle: ValueHandle) -> Result<()> {
        if self.stack.len() < STACK_MAX {
            self.stack.push(handle);

            Ok(())
        } else {
            Err(LoxError::StackOverflow)
        }
    }

    #[inline]
    fn pop(&mut self) -> Result<ValueHandle> {
        self.stack.pop().ok_or(LoxError::StackUnderflow)
    }

    fn pop_number(&mut self) -> Result<f64> {
        let handle = self.pop()?;

        match self.heap.get(&handle) {
            Some(Value::Number(n)) => Ok(*n),
            _ => Err(LoxError::TypeError),
        }
    }

    fn alloc(&mut self, value: Value) -> ValueHandle {
        self.heap.insert(value)
    }
}
