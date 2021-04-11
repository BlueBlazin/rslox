use crate::gc::Handle;
use crate::object::{ObjClosure, ObjString, ObjUpvalue};

#[derive(Clone, Debug)]
pub enum Value {
    Str(ObjString),
    Closure(ObjClosure),
    Upvalue(ObjUpvalue),
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
