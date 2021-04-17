use crate::error::{LoxError, Result};
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

    fn scan_string(&mut self) -> Result<Token> {
        let value = self.scan_until(|c| c == '"');

        self.expect('"').map(|_| Token {
            tok_type: TokenType::Str(value),
            line: self.line,
        })
    }

    fn scan_number(&mut self, c: char) -> Result<Token> {
        let mut value = c.to_string();

        value.push_str(&self.scan_until(|c| !c.is_ascii_digit()));

        if let Some('.') = self.source.peek() {
            value.push(self.source.next().unwrap());

            value.push_str(&self.scan_until(|c| !c.is_ascii_digit()));
        }

        value
            .parse()
            .map_err(|_| LoxError::UnexpectedCharacter)
            .map(|num: f64| Token {
                tok_type: TokenType::Num(num),
                line: self.line,
            })
    }

    fn scan_identifier(&mut self, c: char) -> Option<Result<Token>> {
        let mut value = c.to_string();

        value.push_str(&self.scan_until(|c| !c.is_ascii_alphanumeric()));

        match &value[..] {
            "and" => token!(And, self.line),
            "class" => token!(Class, self.line),
            "else" => token!(Else, self.line),
            "false" => token!(False, self.line),
            "for" => token!(For, self.line),
            "fun" => token!(Fun, self.line),
            "if" => token!(If, self.line),
            "nil" => token!(Nil, self.line),
            "or" => token!(Or, self.line),
            "print" => token!(Print, self.line),
            "return" => token!(Return, self.line),
            "super" => token!(Super, self.line),
            "this" => token!(This, self.line),
            "true" => token!(True, self.line),
            "var" => token!(Var, self.line),
            "while" => token!(While, self.line),
            _ => Some(Ok(Token {
                tok_type: TokenType::Ident(value),
                line: self.line,
            })),
        }
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

    fn scan_until<F>(&mut self, pred: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut value = String::from("");

        loop {
            match self.source.peek() {
                Some(&c) if pred(c) => break,
                Some('\n') => {
                    self.line += 1;
                    self.source.next();
                    value.push('\n');
                }
                Some(&c) => {
                    self.source.next();
                    value.push(c);
                }
                None => break,
            };
        }

        value
    }

    fn expect(&mut self, value: char) -> Result<()> {
        match self.source.next() {
            Some(c) if c == value => Ok(()),
            _ => Err(LoxError::UnexpectedCharacter),
        }
    }
}

impl<'a> Iterator for Scanner<'a> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            self.consume_whitespace();

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
                Some('"') => return Some(self.scan_string()),
                Some(c) if c.is_ascii_digit() => return Some(self.scan_number(c)),
                Some(c) if c.is_ascii_alphabetic() || c == '_' => return self.scan_identifier(c),
                Some(_) => return Some(Err(LoxError::UnexpectedCharacter)),
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        let source = r#"
            fun fib(n) {
                if (n < 2) return n;
                return fib(n - 2) + fib(n - 1);
            }
            
            var start = clock();
            print fib(35) == 9227465;
            print clock() - start;
        "#;

        let scanner = Scanner::new(source.chars());

        println!("{:#?}", scanner.collect::<Vec<_>>());
    }
}
