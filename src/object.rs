use crate::chunk::Chunk;
use crate::value::ValueHandle;
use std::fmt;

#[derive(Clone)]
pub struct ObjString {
    pub value: String,
}

impl fmt::Debug for ObjString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", &self.value)
    }
}

#[derive(Clone)]
pub struct ObjClosure {
    pub arity: usize,
    pub chunk: Chunk,
    // Lox String
    pub name: Option<ValueHandle>,
    pub upvalues: Vec<ValueHandle>,
    pub upvalue_count: usize,
}

impl fmt::Debug for ObjClosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = format!(
            "Lox Function {:?}()\nBytecode of {:?}:\n",
            &self.name.unwrap(),
            &self.name.unwrap()
        );

        // output.push_str(&format!("{:?}---------", &self.chunk));
        output.push_str(&format!("{:?}", &self.chunk));

        write!(f, "{}", output)
    }
}

#[derive(Clone)]
pub struct ObjUpvalue {
    pub location: ValueHandle,
}

impl fmt::Debug for ObjUpvalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", &self.location)
    }
}
