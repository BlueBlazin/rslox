use crate::chunk::Chunk;
use crate::value::{Value, ValueHandle};
use std::collections::HashMap;
use std::fmt;

const EXPAND_CLOSURES: bool = false;

#[derive(Debug)]
pub enum LoxObj {
    Str(ObjString),
    Closure(ObjClosure),
    Upvalue(ObjUpvalue),
    Class(ObjClass),
    Instance(ObjInstance),
    BoundMethod(ObjBoundMethod),
}

// impl fmt::Debug for LoxObj {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             LoxObj::Str(obj) => obj.fmt(f),
//             LoxObj::Closure(obj) => obj.fmt(f),
//             LoxObj::Upvalue(obj) => obj.fmt(f),
//         }
//     }
// }

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
        if !EXPAND_CLOSURES {
            return write!(f, "<Lox Closure {:?}>", &self.name);
        }

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

        // write!(f, "Closure")
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

pub struct ObjClass {
    pub name: String,
    pub methods: HashMap<String, Value>,
    pub is_marked: bool,
}

impl fmt::Debug for ObjClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Class {}>", &self.name)
    }
}

pub struct ObjInstance {
    // Lox Class
    pub class: ValueHandle,
    pub fields: HashMap<String, Value>,
    pub is_marked: bool,
}

impl fmt::Debug for ObjInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Instance of {:?}", &self.class)
    }
}

pub struct ObjBoundMethod {
    // Lox Instance
    pub receiver: Value,
    // Lox Closure
    pub method: ValueHandle,
    pub is_marked: bool,
}

impl fmt::Debug for ObjBoundMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bound Method")
    }
}
