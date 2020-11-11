use crate::chunk::{Chunk, Const};
use crate::codegen::Codegen;
use crate::error::{LoxError, Result};
use crate::object::{LoxObj, ObjString};
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

    pub fn parse(&mut self) -> Result<()> {
        while let Some(_) = self.peek() {
            self.declaration()?;
        }

        Ok(())
    }

    pub fn declaration(&mut self) -> Result<()> {
        match self.peek() {
            Some(TokenType::Var) => self.var_declaration(),
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<()> {
        self.expect(TokenType::Var)?;
        let global = self.parse_variable()?;

        match self.peek() {
            Some(TokenType::Equal) => {
                self.advance()?;
                self.expression()?;
            }
            _ => self.emit_byte(OpCode::Nil as u8),
        }

        self.expect(TokenType::Semicolon)?;

        self.define_variable(global)
    }

    fn parse_variable(&mut self) -> Result<u8> {
        match self.advance()? {
            Some(TokenType::Ident(id)) => self.chunk.add_constant(Const::Str(id)),
            _ => Err(LoxError::UnexpectedToken),
        }
    }

    fn define_variable(&mut self, global: u8) -> Result<()> {
        self.emit_bytes(OpCode::DefineGlobal as u8, global);
        Ok(())
    }

    fn statement(&mut self) -> Result<()> {
        match self.peek() {
            Some(TokenType::Print) => self.print_statement(),
            _ => self.expr_statement(),
        }
    }

    fn print_statement(&mut self) -> Result<()> {
        self.expect(TokenType::Print)?;
        self.expression()?;
        self.expect(TokenType::Semicolon)?;
        self.emit_byte(OpCode::Print as u8);
        Ok(())
    }

    fn expr_statement(&mut self) -> Result<()> {
        self.expression()?;
        self.expect(TokenType::Semicolon)?;
        self.emit_byte(OpCode::Pop as u8);
        Ok(())
    }

    pub fn expression(&mut self) -> Result<()> {
        self.parse_precedence(TokenType::Equal.precedence())
    }

    fn parse_precedence(&mut self, precedence: usize) -> Result<()> {
        let can_assign = precedence <= TokenType::Equal.precedence();

        self.prefix(can_assign)?;

        loop {
            match self.peek() {
                Some(tok_type) if precedence <= tok_type.precedence() => {
                    self.infix()?;
                }
                _ => break,
            }
        }

        match self.peek() {
            Some(TokenType::Equal) if can_assign => Err(LoxError::UnexpectedToken),
            _ => Ok(()),
        }
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
            Some(TokenType::Num(n)) => self.emit_const(Const::Num(n)),
            _ => Err(LoxError::UnexpectedToken),
        }
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

    fn string(&mut self) -> Result<()> {
        match self.advance()? {
            Some(TokenType::Str(s)) => self.emit_const(Const::Str(s)),
            _ => Err(LoxError::UnexpectedToken),
        }
    }

    fn variable(&mut self, can_assign: bool) -> Result<()> {
        let name = self.advance()?.ok_or(LoxError::UnexpectedEOF)?;
        self.named_variable(name, can_assign)
    }

    fn named_variable(&mut self, name: TokenType, can_assign: bool) -> Result<()> {
        match name {
            TokenType::Ident(s) => {
                let arg = self.chunk.add_constant(Const::Str(s))?;

                match self.peek() {
                    Some(TokenType::Equal) if can_assign => {
                        self.advance()?;
                        self.expression()?;
                        self.emit_bytes(OpCode::SetGlobal as u8, arg);
                    }
                    _ => self.emit_bytes(OpCode::GetGlobal as u8, arg),
                }

                Ok(())
            }
            _ => Err(LoxError::UnexpectedToken),
        }
    }

    fn prefix(&mut self, can_assign: bool) -> Result<()> {
        match self.peek().ok_or(LoxError::UnexpectedEOF)? {
            TokenType::LParen => self.grouping(),
            TokenType::Minus | TokenType::Bang => self.unary(),
            TokenType::Num(_) => self.number(),
            TokenType::Nil | TokenType::True | TokenType::False => self.literal(),
            TokenType::Str(_) => self.string(),
            TokenType::Ident(_) => self.variable(can_assign),
            t => unimplemented!("{:?}", t),
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

    fn emit_const(&mut self, value: Const) -> Result<()> {
        let constant = self.chunk.add_constant(value)?;
        self.emit_bytes(OpCode::Constant as u8, constant);
        Ok(())
    }
}
