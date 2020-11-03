use std::any::Any;

#[derive(Debug)]
pub enum LoxObj {
    Str(Box<ObjString>),
}

#[derive(Debug)]
pub struct ObjString {
    pub length: usize,
    pub value: String,
}
