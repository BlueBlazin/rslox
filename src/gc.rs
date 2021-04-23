// Manually managed heap.
// Code is closely adapted from https://github.com/zesterer/broom

use crate::error::{LoxError, Result};
use crate::value::Value;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};

//****************************************************************************
// Handle
//****************************************************************************

pub struct Handle<T: fmt::Debug> {
    pub ptr: *mut T,
}

impl<T: fmt::Debug> fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unsafe { write!(f, "{:?}", &*self.ptr) }
    }
}

impl<T: fmt::Debug> Handle<T> {}

impl<T: fmt::Debug> Copy for Handle<T> {}
impl<T: fmt::Debug> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T: fmt::Debug> PartialEq<Self> for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}
impl<T: fmt::Debug> Eq for Handle<T> {}

impl<T: fmt::Debug> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

//****************************************************************************
// Heap
//****************************************************************************

pub struct Heap<T: fmt::Debug> {
    pub objects: HashSet<Handle<T>>,
}

impl<T: fmt::Debug> Heap<T> {
    pub fn insert(&mut self, value: T) -> Handle<T> {
        let ptr = Box::into_raw(Box::new(value));

        let handle = Handle { ptr };

        self.objects.insert(handle);

        handle
    }

    pub fn contains(&self, handle: &Handle<T>) -> bool {
        self.objects.contains(handle)
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        if self.contains(handle) {
            Some(unsafe { &*handle.ptr })
        } else {
            None
        }
    }

    pub fn get_mut(&self, handle: &Handle<T>) -> Option<&mut T> {
        if self.contains(handle) {
            Some(unsafe { &mut *handle.ptr })
        } else {
            None
        }
    }

    pub fn set(&mut self, handle: &mut Handle<T>, value: T) {
        if self.contains(handle) {
            handle.ptr = Box::into_raw(Box::new(value));
        }
    }

    pub fn remove(&mut self, handle: Handle<T>) {
        let res = self.objects.remove(&handle);
        debug_assert!(!res, "Attempted to remove handle not in heap.");
    }
}

impl<T: fmt::Debug> Drop for Heap<T> {
    fn drop(&mut self) {
        for handle in &self.objects {
            drop(unsafe { Box::from_raw(handle.ptr) });
        }
    }
}

impl<T: fmt::Debug> Default for Heap<T> {
    fn default() -> Self {
        Self {
            objects: HashSet::new(),
        }
    }
}

//****************************************************************************
// GC
//****************************************************************************

// pub trait Gc<T: fmt::Debug> {
//     fn mark(&mut self, handle: Handle<T>) -> Result<()>;
// }

// impl Gc<Value> for Heap<Value> {
//     fn mark(&mut self, handle: Handle<Value>) -> Result<()> {
//         match self
//             .get_mut(&handle)
//             .ok_or(LoxError::_TempDevError("gc mark"))?
//         {
//             Value::Closure(obj) => obj.is_marked = true,
//             Value::Str(obj) => obj.is_marked = true,
//             Value::Upvalue(obj) => obj.is_marked = true,
//             _ => (),
//         }

//         Ok(())
//     }
// }

macro_rules! mark {
    ($obj:expr, $gray_stack:expr, $handle:expr) => {{
        if !$obj.is_marked {
            $obj.is_marked = true;
            $gray_stack.push(*$handle);
        }
    }};
}

pub fn mark_object(
    heap: &Heap<Value>,
    gray_stack: &mut Vec<Handle<Value>>,
    handle: &Handle<Value>,
) -> Result<()> {
    match heap
        .get_mut(&handle)
        .ok_or(LoxError::_TempDevError("gc mark"))?
    {
        Value::Closure(obj) => mark!(obj, gray_stack, handle),
        Value::Str(obj) => mark!(obj, gray_stack, handle),
        Value::Upvalue(obj) => mark!(obj, gray_stack, handle),
        _ => (),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_mut() {
        let mut heap: Heap<Vec<usize>> = Heap::default();

        let handle = heap.insert(vec![1, 2, 3]);

        let a = heap.get_mut(&handle).unwrap();
        let b = heap.get_mut(&handle).unwrap();

        a.push(4);
        b.push(5);

        assert_eq!(heap.get(&handle), Some(&vec![1, 2, 3, 4, 5]));

        heap.remove(handle);

        assert_eq!(heap.contains(&handle), false);

        assert_eq!(heap.get(&handle), None);
    }
}
