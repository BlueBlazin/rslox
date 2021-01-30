use crate::error::{LoxError, Result};
use crate::gc::Heap;
use crate::object::{LoxObj, ObjFunction, ObjString};
use crate::opcodes::OpCode;
use crate::value::{Value, ValueHandle};
use std::collections::HashMap;

const STACK_MAX: usize = 256;
const FRAMES_MAX: usize = 64;

macro_rules! binary_op {
    ($op:tt, $self:expr) => {{
        let b = $self.pop_number()?;
        let a = $self.pop_number()?;

        $self.push_value(Value::Number(a $op b))?;
    }};

    ($op:tt, $self:expr, $type:tt) => {{
        let b = $self.pop_number()?;
        let a = $self.pop_number()?;

        $self.push_value(Value::$type(a $op b))?;
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
    function: ObjFunction,
    ip: usize,
}

impl Vm {
    pub fn new(function: ObjFunction, heap: Heap<Value>) -> Self {
        Self {
            stack: Vec::with_capacity(256),
            heap,
            frames: Vec::with_capacity(FRAMES_MAX),
            globals: HashMap::new(),
            function,
            ip: 0,
        }
    }

    pub fn interpret(&mut self) -> Result<()> {
        self.ip = 0;

        self.run()
    }

    fn run(&mut self) -> Result<()> {
        loop {
            match OpCode::from(self.fetch()) {
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Constant => {
                    let handle = self.fetch_const();

                    self.push(handle)?
                }
                OpCode::Negate => {
                    let value = self.pop_number()?;

                    self.push_value(Value::Number(-value))?;
                }
                OpCode::Add => {
                    let handle_b = self.pop()?;
                    let handle_a = self.pop()?;

                    let b = self.get_value(handle_b)?;
                    let a = self.get_value(handle_a)?;

                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            let sum = *a + *b;

                            self.push_value(Value::Number(sum))?;
                        }
                        (Value::Obj(LoxObj::Str(a)), Value::Obj(LoxObj::Str(b))) => {
                            let mut value = String::from(&a.value);
                            value.push_str(&b.value);

                            self.push_value(Value::Obj(LoxObj::Str(ObjString { value })))?;
                        }
                        _ => return Err(LoxError::TypeError),
                    }
                }
                OpCode::Subtract => binary_op!(-, self),
                OpCode::Multiply => binary_op!(*, self),
                OpCode::Divide => binary_op!(/, self),

                OpCode::Nil => self.push_value(Value::Nil)?,
                OpCode::True => self.push_value(Value::Bool(true))?,
                OpCode::False => self.push_value(Value::Bool(false))?,

                OpCode::Not => {
                    let handle = self.pop()?;

                    let value = self
                        .heap
                        .get(&handle)
                        .ok_or(LoxError::RuntimeError)?
                        .is_falsey();

                    self.push_value(Value::Bool(value))?;
                }
                OpCode::Equal => {
                    let handle_b = self.pop()?;
                    let handle_a = self.pop()?;

                    let b = self.get_value(handle_b)?;
                    let a = self.get_value(handle_a)?;

                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            let cmp = a == b;
                            self.push_value(Value::Bool(cmp))?;
                        }
                        (Value::Obj(LoxObj::Str(a)), Value::Obj(LoxObj::Str(b))) => {
                            let cmp = a.value == b.value;
                            self.push_value(Value::Bool(cmp))?;
                        }
                        _ => return Err(LoxError::TypeError),
                    }
                }
                OpCode::Greater => binary_op!(>, self, Bool),
                OpCode::Less => binary_op!(<, self, Bool),

                OpCode::Print => {
                    let handle = self.pop()?;

                    println!("{:?}", self.get_value(handle));
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

                    if self.heap.get(&handle).unwrap().is_falsey() {
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
            };
        }
    }

    fn fetch_str_const(&mut self) -> Result<String> {
        let handle = self.fetch_const();

        match self.heap.get(&handle) {
            Some(Value::Obj(LoxObj::Str(ObjString { value }))) => Ok(value.clone()),
            _ => Err(LoxError::RuntimeError),
        }
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

    fn push_value(&mut self, value: Value) -> Result<()> {
        let handle = self.alloc(value);

        self.push(handle)
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

    fn get_value(&self, handle: ValueHandle) -> Result<&Value> {
        self.heap.get(&handle).ok_or(LoxError::RuntimeError)
    }

    #[inline]
    fn alloc(&mut self, value: Value) -> ValueHandle {
        self.heap.insert(value)
    }
}
