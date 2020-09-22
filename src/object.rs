use std::any::Any;

pub trait LoxObj: std::fmt::Debug + Any {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, PartialEq)]
pub enum ObjType {
    Str,
}

#[derive(Debug, PartialEq)]
pub struct Obj {
    pub obj_type: ObjType,
}

#[derive(Debug, PartialEq)]
pub struct ObjString {
    pub obj: Obj,
    pub length: usize,
    pub string: String,
}

impl LoxObj for ObjString {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obj_string() {
        let s: Box<dyn LoxObj> = Box::from(ObjString {
            obj: Obj {
                obj_type: ObjType::Str,
            },
            length: 3,
            string: String::from("abc"),
        });

        println!("{:?}", s.as_any().downcast_ref::<ObjString>().unwrap().obj);
    }
}
