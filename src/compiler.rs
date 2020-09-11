use crate::scanner::Scanner;
use std::str::Chars;

pub struct Compiler<'a> {
    scanner: Scanner<'a>,
    line: usize,
}

impl<'a> Compiler<'a> {
    pub fn new(source: Chars<'a>) -> Self {
        Self {
            scanner: Scanner::new(source),
            line: 0,
        }
    }
}
