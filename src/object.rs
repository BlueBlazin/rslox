use crate::chunk::Chunk;
use crate::value::{Value, ValueHandle};
use std::fmt;

pub enum LoxObj {
    Str(ObjString),
    Closure(ObjClosure),
    Upvalue(ObjUpvalue),
}

impl fmt::Debug for LoxObj {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoxObj::Str(obj) => obj.fmt(f),
            LoxObj::Closure(obj) => obj.fmt(f),
            LoxObj::Upvalue(obj) => obj.fmt(f),
        }
    }
}

pub struct ObjString {
    pub value: String,
    pub is_marked: bool,
}

impl fmt::Debug for ObjString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", &self.value)
    }
}

pub struct ObjClosure {
    pub arity: usize,
    pub chunk: Chunk,
    // Lox String
    pub name: Option<ValueHandle>,
    // Lox Upvalues
    pub upvalues: Vec<ValueHandle>,
    pub upvalue_count: usize,
    pub is_marked: bool,
}

impl fmt::Debug for ObjClosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = format!(
            "Lox Function {:?}()\nBytecode of {:?}:\n",
            &self
                .name
                .map(|x| format!("{:?}", x))
                .unwrap_or_else(|| "".to_owned()),
            &self
                .name
                .map(|x| format!("{:?}", x))
                .unwrap_or_else(|| "".to_owned())
        );

        output.push_str(&format!("{:?}", &self.chunk));

        write!(f, "{}", output)
    }
}

pub struct ObjUpvalue {
    pub location: usize,
    pub value: Option<Value>,
    pub is_marked: bool,
}

impl fmt::Debug for ObjUpvalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.location)
    }
}
