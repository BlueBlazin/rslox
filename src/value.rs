use crate::object::LoxObj;
// use broom::prelude::*;
use crate::gc::Handle;

#[derive(Debug)]
pub enum Value {
    Obj(LoxObj),
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
