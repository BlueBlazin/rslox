use crate::chunk::Chunk;
use crate::error::{Internal, LoxError, Result};
use crate::gc::{mark_object, mark_table, Heap};
use crate::object::{LoxObj, ObjClass, ObjClosure, ObjInstance, ObjString, ObjUpvalue};
use crate::opcodes::OpCode;
use crate::value::{Value, ValueHandle};
use std::collections::HashMap;

const FRAMES_MAX: usize = 64;
const STACK_MAX: usize = FRAMES_MAX * 256;
const INITIAL_GC_THRESHOLD: usize = 1024 * 1024;
const GC_HEAP_GROW_FACTOR: usize = 2;

// To force the GC to be called upon every allocation
const DEV_GC_TESTING: bool = true;

const fn lox_obj_size() -> usize {
    std::mem::size_of::<LoxObj>()
}

macro_rules! dprintln {
    ($($arg:tt)*) => ({
        #[cfg(debug_assertions)]
        {
            println!($($arg)*)
        }
    })
}

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

macro_rules! sweep_obj {
    ($obj:expr, $handle:expr, $bytes_freed:expr) => {{
        let is_marked = $obj.is_marked;

        if is_marked {
            $obj.is_marked = false;
        } else {
            dprintln!("Dropping {:?}", $handle);

            $bytes_freed += lox_obj_size();

            drop(unsafe { Box::from_raw($handle.ptr) });
        }

        is_marked
    }};
}

pub struct CallFrame {
    pub closure: ValueHandle,
    pub ip: usize,
    pub fp: usize,
}

pub struct Vm {
    pub stack: Vec<Option<Value>>,
    pub heap: Heap<LoxObj>,
    pub frames: Vec<CallFrame>,
    globals: HashMap<String, Value>,
    sp: usize,
    // TODO: use a BTreeMap instead
    open_upvalues: Vec<(usize, ValueHandle)>,
    gray_stack: Vec<ValueHandle>,
    bytes_allocated: usize,
    next_gc: usize,
}

impl Vm {
    pub fn new(heap: Heap<LoxObj>) -> Self {
        Self {
            stack: vec![None; STACK_MAX],
            heap,
            frames: Vec::with_capacity(FRAMES_MAX),
            globals: HashMap::new(),
            sp: 0,
            open_upvalues: Vec::with_capacity(8),
            gray_stack: Vec::with_capacity(8),
            bytes_allocated: 0,
            next_gc: INITIAL_GC_THRESHOLD,
        }
    }

    pub fn interpret(&mut self, closure: ObjClosure) -> Result<()> {
        // No GC alloc
        let handle = self.heap.insert(LoxObj::Closure(closure));

        // Mark closure so it's not GCd
        mark_object(&self.heap, &mut self.gray_stack, &handle)?;

        let value = Value::Obj(handle);

        self.push(value)?;

        self.call_value(value, 0)?;

        self.run()
    }

    fn run(&mut self) -> Result<()> {
        while let Some(opcode) = self.fetch_opcode() {
            match OpCode::from(*opcode) {
                OpCode::Return => {
                    let value = self.pop()?;

                    let popped_frame = self.frames.pop().unwrap();

                    self.close_upvalues(popped_frame.fp)?;

                    self.sp = popped_frame.fp;

                    self.push(value)?;
                }
                OpCode::Constant => {
                    let value = self.fetch_const();

                    self.push(value)?
                }
                OpCode::Negate => {
                    let n = self.pop_number()?;

                    self.push(Value::Number(-n))?;
                }
                OpCode::Add => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            let sum = a + b;

                            self.push(Value::Number(sum))?;
                        }
                        (Value::Obj(handle_a), Value::Obj(handle_b)) => {
                            let obj_a = self.get_obj(handle_a)?;
                            let obj_b = self.get_obj(handle_b)?;

                            match (obj_a, obj_b) {
                                (LoxObj::Str(a), LoxObj::Str(b)) => {
                                    let mut value = String::from(&a.value);
                                    value.push_str(&b.value);

                                    let lox_val = self.alloc_value(LoxObj::Str(ObjString {
                                        value,
                                        is_marked: false,
                                    }));

                                    self.push(lox_val)?;
                                }
                                _ => return Err(LoxError::TypeError),
                            }
                        }
                        _ => return Err(LoxError::InvalidTypeForAddition),
                    }
                }
                OpCode::Subtract => binary_op!(-, self),
                OpCode::Multiply => binary_op!(*, self),
                OpCode::Divide => binary_op!(/, self),

                OpCode::Nil => self.push(Value::Nil)?,
                OpCode::True => self.push(Value::Bool(true))?,
                OpCode::False => self.push(Value::Bool(false))?,

                OpCode::Not => {
                    let value = self.pop()?.is_falsey();

                    self.push(Value::Bool(value))?;
                }
                OpCode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;

                    match (a, b) {
                        (Value::Number(a), Value::Number(b)) => {
                            let cmp = a.eq(&b);
                            self.push(Value::Bool(cmp))?;
                        }
                        (Value::Obj(handle_a), Value::Obj(handle_b)) => {
                            let obj_a = self.get_obj(handle_a)?;
                            let obj_b = self.get_obj(handle_b)?;

                            match (obj_a, obj_b) {
                                (LoxObj::Str(a), LoxObj::Str(b)) => {
                                    let cmp = a.value == b.value;

                                    self.push(Value::Bool(cmp))?;
                                }
                                _ => return Err(LoxError::TypeError),
                            }
                        }
                        _ => return Err(LoxError::InvalidTypeForEquals),
                    }
                }
                OpCode::Greater => binary_op!(>, self, Bool),
                OpCode::Less => binary_op!(<, self, Bool),

                OpCode::Print => {
                    let value = self.pop()?;
                    println!("{:?}", value);
                }
                OpCode::Pop => {
                    self.pop()?;
                }
                OpCode::DefineGlobal => {
                    let name = self.fetch_str_const()?;
                    let value = self.pop()?;
                    self.globals.insert(name, value);
                }
                OpCode::GetGlobal => {
                    // TODO: explore the possibility of using &'a str instead
                    // for querying the globals hash table.
                    // NOTE: if that is possible, take care to avoid GC cleanup.
                    let name = self.fetch_str_const()?;
                    let value = *self
                        .globals
                        .get(&name)
                        .ok_or(LoxError::InternalError(Internal::GlobalLookupFailure))?;

                    self.push(value)?;
                }
                OpCode::SetGlobal => {
                    let name = self.fetch_str_const()?;

                    if !self.globals.contains_key(&name) {
                        return Err(LoxError::InternalError(Internal::GlobalLookupFailure));
                    }

                    let value = self.peek()?;

                    self.globals.insert(name, value);
                }
                OpCode::GetLocal => {
                    let idx = self.fetch() as usize;
                    let fp = self.current_frame().fp;
                    let value = self.stack[fp + idx].ok_or(LoxError::StackOverflow)?;
                    self.push(value)?;
                }
                OpCode::SetLocal => {
                    let idx = self.fetch() as usize;
                    let value = self.peek()?;
                    let fp = self.current_frame().fp;
                    self.stack[fp + idx] = Some(value);
                }
                OpCode::JumpIfFalse => {
                    let offset = self.fetch16() as usize;

                    let value = self.peek()?;

                    if value.is_falsey() {
                        self.current_frame_mut().ip += offset;
                    }
                }
                OpCode::Jump => {
                    let offset = self.fetch16() as usize;
                    self.current_frame_mut().ip += offset;
                }
                OpCode::Loop => {
                    let offset = self.fetch16() as usize;
                    self.current_frame_mut().ip -= offset;
                }
                OpCode::Call => {
                    let arg_count = self.fetch() as usize;

                    let value =
                        self.stack[self.sp - 1 - arg_count].ok_or(LoxError::StackUnderflow)?;

                    self.call_value(value, arg_count)?;
                }
                OpCode::Closure => {
                    let value = self.fetch_const();
                    let closure_handle = self.get_handle(&value)?;

                    self.push(value)?;

                    let upvalue_count = match self.get_obj(closure_handle)? {
                        LoxObj::Closure(closure) => Ok(closure.upvalue_count),
                        _ => Err(LoxError::_TempDevError(
                            "error in closure upvalue_count match",
                        )),
                    }?;

                    for _ in 0..upvalue_count {
                        let is_local = self.fetch() != 0;
                        let index = self.fetch() as usize;

                        if is_local {
                            let handle = self.capture_upvalue(index);

                            match self.get_obj_mut(closure_handle)? {
                                LoxObj::Closure(closure) => {
                                    closure.upvalues.push(handle);
                                }
                                _ => {
                                    return Err(LoxError::_TempDevError(
                                        "error in closure if is_local",
                                    ))
                                }
                            }
                        } else {
                            let upvalue_handle = self.current_closure()?.upvalues[index];

                            match self.get_obj_mut(closure_handle)? {
                                LoxObj::Closure(closure) => {
                                    closure.upvalues.push(upvalue_handle);
                                }
                                _ => {
                                    return Err(LoxError::_TempDevError(
                                        "error in closure match get_value_mut",
                                    ))
                                }
                            }
                        }
                    }
                }
                OpCode::GetUpvalue => {
                    let idx = self.fetch() as usize;
                    let upvalue_handle = self.current_closure()?.upvalues[idx];

                    match self.get_obj(upvalue_handle)? {
                        LoxObj::Upvalue(upvalue) => {
                            let value = match upvalue.value {
                                Some(value) => value,
                                None => {
                                    self.stack[upvalue.location].ok_or(LoxError::StackOverflow)?
                                }
                            };

                            self.push(value)?;
                        }
                        _ => return Err(LoxError::_TempDevError("get_upvalue")),
                    }
                }
                OpCode::SetUpvalue => {
                    let idx = self.fetch() as usize;
                    let value = self.peek()?;

                    let upvalue_handle = &self.current_closure()?.upvalues[idx];

                    match self
                        .heap
                        .get_mut(upvalue_handle)
                        .ok_or(LoxError::InternalError(Internal::InvalidHandle))?
                    {
                        LoxObj::Upvalue(upvalue) => match upvalue.value {
                            Some(_) => {
                                upvalue.value = Some(value);
                            }
                            None => {
                                self.stack[upvalue.location] = Some(value);
                            }
                        },
                        _ => return Err(LoxError::_TempDevError("set_upvalue")),
                    }
                }
                OpCode::CloseUpvalue => {
                    self.close_upvalues(self.sp - 1)?;
                    self.pop()?;
                }
                OpCode::Class => {
                    let name = self.fetch_str_const()?;

                    let lox_val = self.alloc_value(LoxObj::Class(ObjClass {
                        name,
                        is_marked: false,
                    }));

                    self.push(lox_val)?;
                }
                OpCode::GetProperty => {
                    let name = self.fetch_str_const()?;

                    // pop instance and get object
                    let lox_obj = match self.pop()? {
                        Value::Obj(handle) => self.get_obj(handle),
                        _ => Err(LoxError::_TempDevError("get property")),
                    }?;

                    // get current value of property
                    let value = match lox_obj {
                        LoxObj::Instance(instance) => match instance.fields.get(&name) {
                            Some(value) => Ok(*value),
                            None => Err(LoxError::UndefinedProperty(name)),
                        },
                        _ => Err(LoxError::_TempDevError("get property on non-instance")),
                    }?;

                    // push value onto the stack
                    self.push(value)?;
                }
                OpCode::SetProperty => {
                    let name = self.fetch_str_const()?;

                    // pop new value to be set
                    let value = self.pop()?;

                    // pop instance and get object
                    let lox_obj = match self.pop()? {
                        Value::Obj(handle) => self.get_obj_mut(handle),
                        _ => Err(LoxError::_TempDevError("set property")),
                    }?;

                    // set value of field to new value
                    match lox_obj {
                        LoxObj::Instance(instance) => instance.fields.insert(name, value),
                        _ => return Err(LoxError::_TempDevError("set property on non-instance")),
                    };

                    // push new value onto stack
                    self.push(value)?;
                }
            };
        }

        Ok(())
    }

    fn close_upvalues(&mut self, last: usize) -> Result<()> {
        while let Some((_, handle)) = self.open_upvalues.last() {
            match self
                .heap
                .get_mut(handle)
                .ok_or(LoxError::InternalError(Internal::InvalidHandle))?
            {
                LoxObj::Upvalue(upvalue) => {
                    let location = upvalue.location;

                    if location < last {
                        break;
                    }

                    let value = self.stack[location]
                        .ok_or(LoxError::_TempDevError("close_upvalues StackUnderflow"))?;

                    upvalue.value = Some(value);
                    self.open_upvalues.pop();
                }
                _ => return Err(LoxError::_TempDevError("close_upvalues get_value_mut")),
            }
        }

        Ok(())
    }

    fn capture_upvalue(&mut self, index: usize) -> ValueHandle {
        let location = self.current_frame().fp + index;

        match self
            .open_upvalues
            .binary_search_by_key(&location, |&(i, _)| i)
        {
            Ok(idx) => self
                .open_upvalues
                .get(idx)
                .map(|(_, handle)| *handle)
                .unwrap(),
            Err(idx) => {
                let upvalue_handle = self.alloc(LoxObj::Upvalue(ObjUpvalue {
                    location,
                    value: None,
                    is_marked: false,
                }));

                self.open_upvalues.insert(idx, (location, upvalue_handle));

                upvalue_handle
            }
        }
    }

    fn call_value(&mut self, value: Value, arg_count: usize) -> Result<()> {
        let handle = match value {
            Value::Obj(handle) => handle,
            _ => return Err(LoxError::ValueNotCallable),
        };

        match self.get_obj(handle)? {
            LoxObj::Closure(_) => {
                self.frames.push(CallFrame {
                    closure: handle,
                    ip: 0,
                    fp: self.sp - 1 - arg_count,
                });

                Ok(())
            }
            LoxObj::Class(_) => {
                let lox_val = self.alloc_value(LoxObj::Instance(ObjInstance {
                    class: handle,
                    fields: HashMap::new(),
                    is_marked: false,
                }));

                // TODO: this is not quite right
                self.stack[self.sp - 1 - arg_count] = Some(lox_val);

                Ok(())
            }
            _ => Err(LoxError::ValueNotCallable),
        }
    }

    fn get_handle(&self, value: &Value) -> Result<ValueHandle> {
        match value {
            Value::Obj(handle) => Ok(*handle),
            _ => Err(LoxError::_TempDevError("get_handle")),
        }
    }

    fn fetch_str_const(&mut self) -> Result<String> {
        let value = self.fetch_const();

        match value {
            Value::Obj(handle) => match self.get_obj(handle)? {
                LoxObj::Str(s) => Ok(s.value.clone()),
                _ => Err(LoxError::UnexpectedValue(value)),
            },
            value => Err(LoxError::UnexpectedValue(value)),
        }
    }

    fn fetch16(&mut self) -> u16 {
        let hi = self.fetch();
        let lo = self.fetch();
        (hi as u16) << 8 | (lo as u16)
    }

    #[inline]
    fn current_frame(&self) -> &CallFrame {
        let last = self.frames.len() - 1;
        &self.frames[last]
    }

    #[inline]
    fn current_frame_mut(&mut self) -> &mut CallFrame {
        let last = self.frames.len() - 1;
        &mut self.frames[last]
    }

    fn current_closure(&self) -> Result<&ObjClosure> {
        let handle = self.current_frame().closure;
        match self.get_obj(handle)? {
            LoxObj::Closure(closure) => Ok(closure),
            _ => Err(LoxError::RuntimeError),
        }
    }

    #[inline]
    fn fetch_opcode(&mut self) -> Option<&u8> {
        let frame = self.current_frame_mut();
        let ip = frame.ip;

        frame.ip += 1;

        self.chunk().unwrap().code.get(ip)
    }

    #[inline]
    fn fetch(&mut self) -> u8 {
        let frame = self.current_frame_mut();
        let ip = frame.ip;

        frame.ip += 1;

        self.chunk().unwrap().code[ip]
    }

    #[inline]
    fn fetch_const(&mut self) -> Value {
        let idx = self.fetch() as usize;

        self.chunk().unwrap().constants[idx]
    }

    fn push(&mut self, value: Value) -> Result<()> {
        if self.sp == self.stack.len() {
            Err(LoxError::StackOverflow)
        } else {
            self.stack[self.sp] = Some(value);
            self.sp += 1;
            Ok(())
        }
    }

    fn pop(&mut self) -> Result<Value> {
        if self.sp == 0 {
            return Err(LoxError::StackUnderflow);
        }

        self.sp -= 1;

        self.stack[self.sp]
            .take()
            .ok_or(LoxError::InternalError(Internal::CorruptedStack))
    }

    fn peek(&mut self) -> Result<Value> {
        self.stack[self.sp - 1].ok_or(LoxError::InternalError(Internal::CorruptedStack))
    }

    fn pop_number(&mut self) -> Result<f64> {
        let value = self.pop()?;

        match value {
            Value::Number(n) => Ok(n),
            value => Err(LoxError::UnexpectedValue(value)),
        }
    }

    #[inline]
    fn get_obj(&self, handle: ValueHandle) -> Result<&LoxObj> {
        self.heap
            .get(&handle)
            .ok_or(LoxError::InternalError(Internal::InvalidHandle))
    }

    #[inline]
    fn get_obj_mut(&mut self, handle: ValueHandle) -> Result<&mut LoxObj> {
        self.heap
            .get_mut(&handle)
            .ok_or(LoxError::InternalError(Internal::InvalidHandle))
    }

    fn update_bytes_allocated(&mut self) {
        self.bytes_allocated += lox_obj_size();

        if self.bytes_allocated > self.next_gc {
            self.collect_garbage().unwrap();
        }
    }

    fn alloc(&mut self, obj: LoxObj) -> ValueHandle {
        if DEV_GC_TESTING && cfg!(debug_assertions) {
            println!("Allocing {:?}", &obj);
            self.collect_garbage().unwrap();
        } else {
            self.update_bytes_allocated();
        }

        self.heap.insert(obj)
    }

    fn alloc_value(&mut self, obj: LoxObj) -> Value {
        let handle = self.alloc(obj);

        Value::Obj(handle)
    }

    fn mark_roots(&mut self) -> Result<()> {
        dprintln!("mark roots start");

        dprintln!("marking stack variables");
        // mark stack variables
        for i in 0..self.sp {
            match &self.stack[i] {
                Some(value) => {
                    if let Value::Obj(handle) = value {
                        mark_object(&self.heap, &mut self.gray_stack, handle)?;
                    }
                }
                None => break,
            }
        }

        dprintln!("marking closure objects");
        // mark closure objects
        for frame in &self.frames {
            mark_object(&self.heap, &mut self.gray_stack, &frame.closure)?;
        }

        dprintln!("marking upvalues");
        // mark upvalues
        for (_, handle) in &self.open_upvalues {
            mark_object(&self.heap, &mut self.gray_stack, handle)?;
        }

        dprintln!("marking globals");
        // mark globals
        // self.mark_table()?;
        mark_table(&self.heap, &mut self.gray_stack, &self.globals)?;

        dprintln!("mark roots end");

        Ok(())
    }

    fn trace_references(&mut self) -> Result<()> {
        while let Some(handle) = self.gray_stack.pop() {
            self.blacken_object(handle)?;
        }

        Ok(())
    }

    /// Rslox specific tracing for lox objects.
    fn blacken_object(&mut self, handle: ValueHandle) -> Result<()> {
        let value = self
            .heap
            .get(&handle)
            .ok_or(LoxError::InternalError(Internal::InvalidHandle))?;

        match value {
            LoxObj::Str(_) => (),
            LoxObj::Closure(obj) => {
                if let Some(name_handle) = &obj.name {
                    mark_object(&self.heap, &mut self.gray_stack, name_handle)?;
                }

                for value in &obj.chunk.constants {
                    if let Value::Obj(handle) = value {
                        mark_object(&self.heap, &mut self.gray_stack, handle)?;
                    }
                }

                for upvalue_handle in &obj.upvalues {
                    mark_object(&self.heap, &mut self.gray_stack, upvalue_handle)?;
                }
            }
            LoxObj::Upvalue(obj) => match &obj.value {
                Some(Value::Obj(upvalue_handle)) => {
                    mark_object(&self.heap, &mut self.gray_stack, upvalue_handle)?;
                }
                Some(_) => return Err(LoxError::_TempDevError("expected upvalue obj")),
                None => (),
            },
            LoxObj::Class(_) => (),
            LoxObj::Instance(obj) => {
                mark_object(&self.heap, &mut self.gray_stack, &obj.class)?;

                mark_table(&self.heap, &mut self.gray_stack, &obj.fields)?;
            }
        }

        Ok(())
    }

    fn sweep(&mut self) {
        let mut bytes_freed = 0;

        self.heap.objects = self
            .heap
            .objects
            .iter()
            .filter(|&handle| match self.heap.get_mut(handle) {
                Some(LoxObj::Closure(obj)) => sweep_obj!(obj, handle, bytes_freed),
                Some(LoxObj::Str(obj)) => sweep_obj!(obj, handle, bytes_freed),
                Some(LoxObj::Upvalue(obj)) => sweep_obj!(obj, handle, bytes_freed),
                Some(LoxObj::Class(obj)) => sweep_obj!(obj, handle, bytes_freed),
                Some(LoxObj::Instance(obj)) => sweep_obj!(obj, handle, bytes_freed),
                None => panic!(), // TODO: change this to an error instead
            })
            .copied()
            .collect();

        if !(DEV_GC_TESTING && cfg!(debug_assertions)) {
            self.bytes_allocated -= bytes_freed;
        }
    }

    fn collect_garbage(&mut self) -> Result<()> {
        dprintln!("gc begin");

        self.mark_roots()?;

        self.trace_references()?;

        self.sweep();

        self.next_gc = self.bytes_allocated * GC_HEAP_GROW_FACTOR;

        dprintln!("gc end");

        Ok(())
    }

    #[inline]
    fn chunk(&mut self) -> Result<&Chunk> {
        let handle = self.current_frame().closure;

        match self.get_obj(handle) {
            Ok(LoxObj::Closure(f)) => Ok(&f.chunk),
            _ => Err(LoxError::RuntimeError),
        }
    }
}
