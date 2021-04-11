use crate::chunk::Chunk;
use crate::value::ValueHandle;
use std::fmt;

// #[derive(Debug, Clone)]
// pub enum LoxObj {
//     Str(ObjString),
//     Fun(ObjFunction),
// }

#[derive(Clone, Debug)]
pub struct ObjString {
    pub value: String,
}

#[derive(Clone, Debug)]
pub struct ObjClosure {
    pub arity: usize,
    pub chunk: Chunk,
    // Lox String
    pub name: Option<ValueHandle>,
    pub upvalues: Vec<ValueHandle>,
    pub upvalue_count: usize,
}

// impl fmt::Debug for ObjClosure {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "ObjClosure {:?}", self.name.unwrap())
//     }
// }

#[derive(Clone, Debug)]
pub struct ObjUpvalue {
    pub location: ValueHandle,
}
