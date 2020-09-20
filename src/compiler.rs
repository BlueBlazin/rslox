use crate::chunk::Chunk;
use crate::codegen::Codegen;
use crate::error::{LoxError, Result};
use crate::opcodes::OpCode;
use crate::scanner::Scanner;
use crate::token::{Token, TokenType};
use crate::value::Value;
use std::iter::Peekable;
use std::str::Chars;

pub struct Compiler<'a> {
    scanner: Peekable<Scanner<'a>>,
    pub line: usize,
    pub chunk: Chunk,
}

impl<'a> Compiler<'a> {
    pub fn new(source: Chars<'a>) -> Self {
        Self {
            scanner: Scanner::new(source).peekable(),
            line: 0,
            chunk: Chunk::new(String::from("0")),
        }
    }

    pub fn expression(&mut self) -> Result<()> {
        self.parse_precedence(TokenType::Equal.precedence())
    }

    fn parse_precedence(&mut self, precedence: usize) -> Result<()> {
        self.prefix()?;

        loop {
            match self.peek() {
                Some(tok_type) if precedence <= tok_type.precedence() => {
                    self.infix()?;
                }
                _ => break,
            }
        }

        Ok(())
    }

    fn binary(&mut self) -> Result<()> {
        let op = self.advance()?.ok_or(LoxError::UnexpectedEOF)?;

        self.parse_precedence(op.precedence())?;

        match op {
            TokenType::Plus => self.emit_byte(OpCode::Add as u8),
            TokenType::Minus => self.emit_byte(OpCode::Subtract as u8),
            TokenType::Star => self.emit_byte(OpCode::Multiply as u8),
            TokenType::Slash => self.emit_byte(OpCode::Divide as u8),
            TokenType::BangEq => self.emit_bytes(OpCode::Equal as u8, OpCode::Not as u8),
            TokenType::EqualEq => self.emit_byte(OpCode::Equal as u8),
            TokenType::Greater => self.emit_byte(OpCode::Greater as u8),
            TokenType::GreaterEq => self.emit_bytes(OpCode::Less as u8, OpCode::Not as u8),
            TokenType::Less => self.emit_byte(OpCode::Less as u8),
            TokenType::LessEq => self.emit_bytes(OpCode::Greater as u8, OpCode::Not as u8),
            _ => return Err(LoxError::UnexpectedToken),
        }

        Ok(())
    }

    fn unary(&mut self) -> Result<()> {
        let op = self.advance()?.ok_or(LoxError::UnexpectedEOF)?;

        self.expression()?;

        match op {
            TokenType::Minus => self.emit_byte(OpCode::Negate as u8),
            TokenType::Bang => self.emit_byte(OpCode::Not as u8),
            _ => return Err(LoxError::UnexpectedToken),
        }

        Ok(())
    }

    fn grouping(&mut self) -> Result<()> {
        self.expect(TokenType::LParen)?;

        self.expression()?;

        self.expect(TokenType::RParen).map(|_| ())
    }

    fn number(&mut self) -> Result<()> {
        match self.advance()? {
            Some(TokenType::Num(n)) => self.emit_const(n),
            _ => return Err(LoxError::UnexpectedToken),
        }

        Ok(())
    }

    fn literal(&mut self) -> Result<()> {
        match self.advance()? {
            Some(TokenType::Nil) => self.emit_byte(OpCode::Nil as u8),
            Some(TokenType::True) => self.emit_byte(OpCode::True as u8),
            Some(TokenType::False) => self.emit_byte(OpCode::False as u8),
            _ => unreachable!(),
        }

        Ok(())
    }

    fn prefix(&mut self) -> Result<()> {
        match self.peek().ok_or(LoxError::UnexpectedEOF)? {
            TokenType::LParen => self.grouping(),
            TokenType::Minus | TokenType::Bang => self.unary(),
            TokenType::Num(_) => self.number(),
            TokenType::Nil | TokenType::True | TokenType::False => self.literal(),
            _ => unimplemented!(),
        }
    }

    fn infix(&mut self) -> Result<()> {
        match self.peek().ok_or(LoxError::UnexpectedEOF)? {
            TokenType::Plus
            | TokenType::Minus
            | TokenType::Star
            | TokenType::Slash
            | TokenType::BangEq
            | TokenType::EqualEq
            | TokenType::Less
            | TokenType::LessEq
            | TokenType::Greater
            | TokenType::GreaterEq => self.binary(),
            _ => unimplemented!(),
        }
    }

    fn advance(&mut self) -> Result<Option<TokenType>> {
        match self.scanner.next() {
            Some(Ok(Token { line, tok_type })) => {
                self.line = line;
                Ok(Some(tok_type))
            }
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    fn peek(&mut self) -> Option<&TokenType> {
        match self.scanner.peek() {
            Some(Ok(Token { tok_type, .. })) => Some(tok_type),
            _ => None,
        }
    }

    fn expect(&mut self, expected_type: TokenType) -> Result<TokenType> {
        match self.advance()? {
            Some(tok_type) if tok_type == expected_type => Ok(tok_type),
            _ => Err(LoxError::UnexpectedToken),
        }
    }
}

impl<'a> Codegen for Compiler<'a> {
    fn emit_byte(&mut self, value: u8) {
        self.chunk.write(value, self.line);
    }

    fn emit_const(&mut self, value: f64) {
        let constant = self.chunk.add_constant(value);
        self.emit_bytes(OpCode::Constant as u8, constant);
    }
}
