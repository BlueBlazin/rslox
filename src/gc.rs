// Manually managed heap.
// Code is closely adapted from https://github.com/zesterer/broom

use std::collections::HashSet;
use std::hash::{Hash, Hasher};

//****************************************************************************
// Handle
//****************************************************************************

#[derive(Debug)]
pub struct Handle<T> {
    ptr: *mut T,
}

impl<T> Handle<T> {}

impl<T> Copy for Handle<T> {}
impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T> PartialEq<Self> for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}
impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state);
    }
}

//****************************************************************************
// Heap
//****************************************************************************

pub struct Heap<T> {
    pub objects: HashSet<Handle<T>>,
}

impl<T> Heap<T> {
    pub fn new() -> Self {
        Self {
            objects: HashSet::new(),
        }
    }

    pub fn insert(&mut self, object: T) -> Handle<T> {
        let ptr = Box::into_raw(Box::new(object));

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

    pub fn remove(&mut self, handle: Handle<T>) {
        let res = self.objects.remove(&handle);
        debug_assert!(!res, "Attempted to remove handle not in heap.");
    }
}

impl<T> Drop for Heap<T> {
    fn drop(&mut self) {
        for handle in &self.objects {
            drop(unsafe { Box::from_raw(handle.ptr) });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_mut() {
        let mut heap: Heap<Vec<usize>> = Heap::new();

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
