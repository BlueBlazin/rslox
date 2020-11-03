use crate::object::LoxObj;
use broom::prelude::*;

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

impl Trace<Self> for Value {
    fn trace(&self, tracer: &mut Tracer<Self>) {
        match self {
            Value::Bool(_) | Value::Number(_) | Value::Nil => (),
            Value::Obj(obj) => unimplemented!(),
        }
    }
}
