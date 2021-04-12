use crate::gc::Handle;
use crate::object::{ObjClosure, ObjString, ObjUpvalue};
use std::fmt;

#[derive(Clone)]
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

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Str(obj_string) => write!(f, "{:?}", obj_string),
            Value::Closure(obj_closure) => write!(f, "{:?}", obj_closure),
            Value::Upvalue(obj_upvalue) => write!(f, "{:?}", obj_upvalue),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::Nil => write!(f, "nil"),
        }
    }
}

pub type ValueHandle = Handle<Value>;
