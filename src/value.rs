use crate::gc::Handle;
use crate::object::LoxObj;
use std::fmt;

pub type ValueHandle = Handle<LoxObj>;

#[derive(Copy, Clone)]
pub enum Value {
    Obj(ValueHandle),
    Bool(bool),
    Number(f64),
    Nil,
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        matches!(self, Value::Nil | Value::Bool(false))
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Obj(handle) => write!(f, "{:?}", handle),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::Nil => write!(f, "nil"),
        }
    }
}
