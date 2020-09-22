use crate::object::LoxObj;
use std::any::Any;
use std::fmt;

pub enum Value {
    Obj(Box<dyn LoxObj>),
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
            Value::Obj(obj) => write!(f, "{:?}", *obj),
            Value::Bool(value) => write!(f, "{}", value),
            Value::Number(value) => write!(f, "{}", value),
            Value::Nil => write!(f, "Nil"),
        }
    }
}

// pub trait LoxObj: std::fmt::Debug {}

// #[derive(Debug, PartialEq)]
// pub enum ObjType {
//     Str,
// }

// #[derive(Debug, PartialEq)]
// pub struct Obj {
//     pub obj_type: ObjType,
// }

// #[derive(Debug, PartialEq)]
// pub struct ObjString {
//     pub obj: Obj,
//     pub length: usize,
//     pub chars: Vec<char>,
// }

// impl LoxObj for ObjString {}
