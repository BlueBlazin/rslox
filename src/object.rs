use crate::chunk::Chunk;
use crate::value::ValueHandle;

#[derive(Debug)]
pub enum LoxObj {
    Str(ObjString),
    Fun(ObjFunction),
}

#[derive(Debug)]
pub struct ObjString {
    pub value: String,
}

#[derive(Debug)]
pub struct ObjFunction {
    pub arity: usize,
    pub chunk: Chunk,
    // Lox String
    pub name: Option<ValueHandle>,
}
