use crate::object::LoxObj;
use std::any::Any;

#[derive(Debug)]
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
