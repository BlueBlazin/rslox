use crate::chunk::Chunk;
use crate::value::ValueHandle;

// #[derive(Debug, Clone)]
// pub enum LoxObj {
//     Str(ObjString),
//     Fun(ObjFunction),
// }

#[derive(Debug, Clone)]
pub struct ObjString {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ObjClosure {
    pub arity: usize,
    pub chunk: Chunk,
    // Lox String
    pub name: Option<ValueHandle>,
    pub upvalues: Vec<ValueHandle>,
    pub upvalue_count: usize,
}

#[derive(Debug, Clone)]
pub struct ObjUpvalue {
    location: ValueHandle,
}
