use crate::error::Result;
use crate::token::{Token, TokenType};
use std::iter::{Iterator, Peekable};
use std::str::Chars;

macro_rules! token {
    ($type:tt, $line:expr) => {
        Some(Ok(Token {
            tok_type: TokenType::$type,
            line: $line,
        }))
    };
}

macro_rules! consume_and_token {
    ($type:tt, $line:expr, $self:expr) => {{
        $self.source.next();
        token!($type, $line)
    }};
}

pub struct Scanner<'a> {
    source: Peekable<Chars<'a>>,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: Chars<'a>) -> Self {
        Scanner {
            source: source.peekable(),
            line: 0,
        }
    }

    fn consume_string(&mut self) -> Option<Result<Token>> {
        unimplemented!()
    }

    fn consume_whitespace(&mut self) {
        loop {
            match self.source.peek() {
                Some(' ') | Some('\t') | Some('\r') => self.source.next(),
                Some('\n') => {
                    self.line += 1;
                    self.source.next()
                }
                _ => break,
            };
        }
    }

    fn scan_comment(&mut self) {
        loop {
            match self.source.peek() {
                None | Some('\n') => break,
                _ => self.source.next(),
            };
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_whitespace();

        loop {
            match self.source.next() {
                Some('(') => return token!(LParen, self.line),
                Some(')') => return token!(RParen, self.line),
                Some('{') => return token!(LBrace, self.line),
                Some('}') => return token!(RBrace, self.line),
                Some(';') => return token!(Semicolon, self.line),
                Some(',') => return token!(Comma, self.line),
                Some('.') => return token!(Dot, self.line),
                Some('-') => return token!(Minus, self.line),
                Some('+') => return token!(Plus, self.line),
                Some('*') => return token!(Star, self.line),
                Some('/') => match self.source.peek() {
                    Some('/') => {
                        self.source.next();
                        self.scan_comment()
                    }
                    _ => return token!(Slash, self.line),
                },
                Some('!') => match self.source.peek() {
                    Some('=') => return consume_and_token!(BangEq, self.line, self),
                    _ => return token!(Bang, self.line),
                },
                Some('=') => match self.source.peek() {
                    Some('=') => return consume_and_token!(EqualEq, self.line, self),
                    _ => return token!(Equal, self.line),
                },
                Some('<') => match self.source.peek() {
                    Some('=') => return consume_and_token!(LessEq, self.line, self),
                    _ => return token!(Less, self.line),
                },
                Some('>') => match self.source.peek() {
                    Some('=') => return consume_and_token!(GreaterEq, self.line, self),
                    _ => return token!(Greater, self.line),
                },
                Some('"') => return self.consume_string(),
                _ => unimplemented!(),
            }
        }
    }
}
