use crate::chunk::Chunk;
use std::any::Any;

#[derive(Debug)]
pub enum LoxObj {
    Str(Box<ObjString>),
    Fun(Box<ObjFunction>),
}

#[derive(Debug)]
pub struct ObjString {
    pub length: usize,
    pub value: String,
}

#[derive(Debug)]
pub struct ObjFunction {
    pub arity: usize,
    pub chunk: Chunk,
    pub name: ObjString,
}
