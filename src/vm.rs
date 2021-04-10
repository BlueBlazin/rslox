use crate::chunk::Chunk;
use crate::error::{Internal, LoxError, Result};
use crate::gc::Heap;
use crate::object::{ObjClosure, ObjString};
use crate::opcodes::OpCode;
use crate::value::{Value, ValueHandle};
use std::collections::HashMap;

const FRAMES_MAX: usize = 64;
const STACK_MAX: usize = FRAMES_MAX * 256;

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
    pub closure: ValueHandle,
    pub ip: usize,
    pub fp: usize,
}

pub struct Vm {
    pub stack: Vec<Option<ValueHandle>>,
    pub heap: Heap<Value>,
    pub frames: Vec<CallFrame>,
    globals: HashMap<String, ValueHandle>,
    sp: usize,
}

impl Vm {
    pub fn new(heap: Heap<Value>) -> Self {
        Self {
            stack: vec![None; STACK_MAX],
            heap,
            frames: Vec::with_capacity(FRAMES_MAX),
            globals: HashMap::new(),
            sp: 0,
        }
    }

    pub fn interpret(&mut self, function: ObjClosure) -> Result<()> {
        let handle = self.alloc(Value::Closure(function));

        self.push(handle)?;

        self.call_value(handle, 0)?;

        self.run()
    }

    fn run(&mut self) -> Result<()> {
        while let Some(opcode) = self.fetch_opcode() {
            match OpCode::from(*opcode) {
                OpCode::Return => {
                    let handle = self.pop()?;

                    let popped_frame = self.frames.pop().unwrap();

                    self.sp = popped_frame.fp;

                    self.push(handle)?;
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
                        (Value::Str(a), Value::Str(b)) => {
                            let mut value = String::from(&a.value);
                            value.push_str(&b.value);

                            self.push_value(Value::Str(ObjString { value }))?;
                        }
                        _ => return Err(LoxError::InvalidTypeForAddition),
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
                        .ok_or(LoxError::InternalError(Internal::InvalidHandle))?
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
                        (Value::Str(a), Value::Str(b)) => {
                            let cmp = a.value == b.value;
                            self.push_value(Value::Bool(cmp))?;
                        }
                        _ => return Err(LoxError::InvalidTypeForEquals),
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
                    // TODO: explore the possibility of using &'a str instead
                    // for querying the globals hash table.
                    // NOTE: if that is possible, take care to avoid GC cleanup.
                    let name = self.fetch_str_const()?;
                    let handle = *self
                        .globals
                        .get(&name)
                        .ok_or(LoxError::InternalError(Internal::GlobalLookupFailure))?;

                    self.push(handle)?;
                }
                OpCode::SetGlobal => {
                    let name = self.fetch_str_const()?;

                    if !self.globals.contains_key(&name) {
                        return Err(LoxError::InternalError(Internal::GlobalLookupFailure));
                    }

                    let handle = self.peek()?;

                    self.globals.insert(name, handle);
                }
                OpCode::GetLocal => {
                    let idx = self.fetch() as usize;
                    let fp = self.current_frame().fp;
                    let handle = self.stack[fp + idx].ok_or(LoxError::StackOverflow)?;
                    self.push(handle)?;
                }
                OpCode::SetLocal => {
                    let idx = self.fetch() as usize;
                    let handle = self.peek()?;
                    let fp = self.current_frame().fp;
                    self.stack[fp + idx] = Some(handle);
                }
                OpCode::JumpIfFalse => {
                    let offset = self.fetch16() as usize;

                    let handle = self.peek()?;

                    if self.get_value(handle).unwrap().is_falsey() {
                        self.current_frame().ip += offset;
                    }
                }
                OpCode::Jump => {
                    let offset = self.fetch16() as usize;
                    self.current_frame().ip += offset;
                }
                OpCode::Loop => {
                    let offset = self.fetch16() as usize;
                    self.current_frame().ip -= offset;
                }
                OpCode::Call => {
                    let arg_count = self.fetch() as usize;

                    let handle =
                        self.stack[self.sp - 1 - arg_count].ok_or(LoxError::StackUnderflow)?;

                    self.call_value(handle, arg_count)?;
                }
                OpCode::Closure => {
                    let handle = self.fetch_const();

                    self.push(handle)?
                }
                _ => unimplemented!(),
            };
        }

        return Ok(());
    }

    fn call_value(&mut self, handle: ValueHandle, arg_count: usize) -> Result<()> {
        match self.get_value(handle)? {
            Value::Closure(_) => {
                self.frames.push(CallFrame {
                    closure: handle,
                    ip: 0,
                    fp: self.sp - 1 - arg_count,
                });

                Ok(())
            }
            _ => Err(LoxError::ValueNotCallable),
        }
    }

    fn fetch_str_const(&mut self) -> Result<String> {
        let handle = self.fetch_const();

        match self.get_value(handle)? {
            Value::Str(ObjString { value }) => Ok(value.clone()),
            value => Err(LoxError::UnexpectedValue(value.clone())),
        }
    }

    fn fetch16(&mut self) -> u16 {
        let hi = self.fetch();
        let lo = self.fetch();
        (hi as u16) << 8 | (lo as u16)
    }

    #[inline]
    fn current_frame(&mut self) -> &mut CallFrame {
        let last = self.frames.len() - 1;
        &mut self.frames[last]
    }

    #[inline]
    fn fetch_opcode(&mut self) -> Option<&u8> {
        let frame = self.current_frame();
        let ip = frame.ip;

        frame.ip += 1;

        self.chunk().unwrap().code.get(ip)
    }

    #[inline]
    fn fetch(&mut self) -> u8 {
        let frame = self.current_frame();
        let ip = frame.ip;

        frame.ip += 1;

        self.chunk().unwrap().code[ip]
    }

    #[inline]
    fn fetch_const(&mut self) -> ValueHandle {
        let idx = self.fetch() as usize;

        self.chunk().unwrap().constants[idx]
    }

    fn push(&mut self, handle: ValueHandle) -> Result<()> {
        if self.sp == self.stack.len() {
            Err(LoxError::StackOverflow)
        } else {
            self.stack[self.sp] = Some(handle);
            self.sp += 1;
            Ok(())
        }
    }

    fn push_value(&mut self, value: Value) -> Result<()> {
        let handle = self.alloc(value);

        self.push(handle)
    }

    fn pop(&mut self) -> Result<ValueHandle> {
        if self.sp == 0 {
            return Err(LoxError::StackUnderflow);
        }

        self.sp -= 1;

        self.stack[self.sp]
            .take()
            .ok_or(LoxError::InternalError(Internal::CorruptedStack))
    }

    fn peek(&mut self) -> Result<ValueHandle> {
        self.stack[self.sp - 1].ok_or(LoxError::InternalError(Internal::CorruptedStack))
    }

    fn pop_number(&mut self) -> Result<f64> {
        let handle = self.pop()?;

        match self.get_value(handle)? {
            Value::Number(n) => Ok(*n),
            value => Err(LoxError::UnexpectedValue(value.clone())),
        }
    }

    #[inline]
    fn get_value(&self, handle: ValueHandle) -> Result<&Value> {
        self.heap
            .get(&handle)
            .ok_or(LoxError::InternalError(Internal::InvalidHandle))
    }

    #[inline]
    fn get_value_mut(&mut self, handle: ValueHandle) -> Result<&mut Value> {
        self.heap
            .get_mut(&handle)
            .ok_or(LoxError::InternalError(Internal::InvalidHandle))
    }

    #[inline]
    fn alloc(&mut self, value: Value) -> ValueHandle {
        self.heap.insert(value)
    }

    #[inline]
    fn chunk(&mut self) -> Result<&Chunk> {
        let handle = self.current_frame().closure;

        match self.get_value(handle) {
            Ok(Value::Closure(f)) => Ok(&f.chunk),
            _ => Err(LoxError::RuntimeError),
        }
    }
}
