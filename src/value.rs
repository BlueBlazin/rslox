use crate::gc::Handle;
use crate::object::{ObjClosure, ObjFunction, ObjString};

#[derive(Debug, Clone)]
pub enum Value {
    Str(ObjString),
    Closure(ObjClosure),
    Fun(ObjFunction),
    Bool(bool),
    Number(f64),
    Nil,
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::Nil | Value::Bool(false) => true,
            _ => false,
        }
    }
}

pub type ValueHandle = Handle<Value>;
