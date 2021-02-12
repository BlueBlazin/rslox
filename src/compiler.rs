use crate::chunk::Chunk;
use crate::codegen::Codegen;
use crate::error::{LoxError, Result};
use crate::gc::Heap;
use crate::object::{LoxObj, ObjFunction, ObjString};
use crate::opcodes::OpCode;
use crate::scanner::Scanner;
use crate::token::{Token, TokenType};
use crate::value::{Value, ValueHandle};
use std::iter::Peekable;
use std::mem;
use std::str::Chars;

struct Local {
    name: String,
    depth: isize,
}

// enum FunctionType {
//     Function,
//     Script,
// }

pub struct Compiler<'a> {
    scanner: Peekable<Scanner<'a>>,
    pub function: ObjFunction,
    // pub functions: Vec<ObjFunction>,
    locals: Vec<Local>,
    scope_depth: isize,
    pub line: usize,
    pub heap: Heap<Value>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: Chars<'a>, heap: Heap<Value>) -> Self {
        // NOTE: I don't think we need to worry about GC here. Still, be mindful.
        let function = ObjFunction {
            arity: 0,
            chunk: Chunk::new(String::from("main")),
            name: None,
        };

        let mut locals = Vec::with_capacity(std::u8::MAX as usize + 1);

        locals.push(Local {
            name: String::from(""),
            depth: 0,
        });

        Self {
            scanner: Scanner::new(source).peekable(),
            // functions: vec![function],
            function,
            locals,
            scope_depth: 0,
            line: 0,
            heap,
        }
    }

    // pub fn from_scanner(scanner: Peekable<Scanner<'a>>, heap: Heap<Value>) -> Self {
    //     let function = ObjFunction {
    //         arity: 0,
    //         chunk: Chunk::new(String::from("main")),
    //         name: None,
    //     };

    //     let mut locals = Vec::with_capacity(std::u8::MAX as usize + 1);

    //     locals.push(Local {
    //         name: String::from(""),
    //         depth: 0,
    //     });

    //     Self {
    //         scanner: Some(scanner),
    //         function,
    //         fun_type: FunctionType::Function,
    //         locals,
    //         scope_depth: 0,
    //         line: 0,
    //         heap: Some(heap),
    //     }
    // }

    pub fn compile(mut self) -> Result<()> {
        self.parse()
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
            Some(TokenType::Fun) => self.fun_declaration(),
            _ => self.statement(),
        }
    }

    fn fun_declaration(&mut self) -> Result<()> {
        self.expect(TokenType::Fun)?;

        let (global, name) = self.parse_function_name()?;

        self.mark_initialized();

        self.function(name)?;

        self.define_variable(global)
    }

    fn function(&mut self, name: String) -> Result<()> {
        let function_obj = self.with_function_ctx(name, &mut |this| {
            this.begin_scope();

            this.parse_parameters()?;

            this.block()
        })?;

        let handle = self.heap.insert(Value::Obj(LoxObj::Fun(function_obj)));

        self.emit_const(handle)
    }

    fn parse_parameters(&mut self) -> Result<()> {
        self.expect(TokenType::LParen)?;

        loop {
            match self.peek() {
                Some(TokenType::RParen) | None => break,
                _ => {
                    self.function.arity += 1;

                    if self.function.arity > 255 {
                        return Err(LoxError::CompileError);
                    }

                    let param_const = self.parse_variable()?;

                    self.define_variable(param_const)?;

                    match self.peek() {
                        Some(TokenType::RParen) | None => (),
                        _ => {
                            self.expect(TokenType::Comma)?;
                        }
                    }
                }
            }
        }

        self.expect(TokenType::RParen)?;

        Ok(())
    }

    fn var_declaration(&mut self) -> Result<()> {
        self.expect(TokenType::Var)?;

        // const_idx is the location in the constants array
        // where the variable name (its handle) will be stored
        let const_idx = self.parse_variable()?;

        match self.peek() {
            Some(TokenType::Equal) => {
                self.advance()?;
                self.expression()?;
            }
            _ => self.emit_byte(OpCode::Nil as u8),
        }

        self.expect(TokenType::Semicolon)?;

        self.define_variable(const_idx)
    }

    fn parse_function_name(&mut self) -> Result<(u8, String)> {
        match self.advance()? {
            Some(TokenType::Ident(id)) => {
                self.declare_variable(id.clone())?;

                if self.scope_depth > 0 {
                    Ok((0, id))
                } else {
                    let handle = self.make_string(id.clone())?;

                    Ok((self.chunk().add_constant(handle)?, id))
                }
            }
            token => Err(LoxError::UnexpectedToken(token)),
        }
    }

    fn parse_variable(&mut self) -> Result<u8> {
        match self.advance()? {
            Some(TokenType::Ident(id)) => {
                self.declare_variable(id.clone())?;

                if self.scope_depth > 0 {
                    Ok(0)
                } else {
                    let handle = self.make_string(id)?;

                    self.chunk().add_constant(handle)
                }
            }
            token => Err(LoxError::UnexpectedToken(token)),
        }
    }

    fn declare_variable(&mut self, name: String) -> Result<()> {
        // variable is global
        if self.scope_depth == 0 {
            return Ok(());
        }

        // variable is local
        for local in self.locals.iter().rev() {
            if local.depth != -1 && local.depth < self.scope_depth {
                break;
            }

            if &name == &local.name {
                return Err(LoxError::CompileError);
            }
        }

        self.add_local(name)
    }

    fn add_local(&mut self, name: String) -> Result<()> {
        if self.locals.len() == 256 {
            return Err(LoxError::TooManyLocalVariables);
        }

        self.locals.push(Local { name, depth: -1 });

        Ok(())
    }

    fn define_variable(&mut self, const_idx: u8) -> Result<()> {
        if self.scope_depth <= 0 {
            self.emit_bytes(OpCode::DefineGlobal as u8, const_idx);
        } else {
            self.mark_initialized();
        }

        Ok(())
    }

    fn mark_initialized(&mut self) {
        if self.scope_depth != 0 {
            let len = self.locals.len();
            self.locals[len - 1].depth = self.scope_depth;
        }
    }

    fn statement(&mut self) -> Result<()> {
        dbg!("statement");
        match self.peek() {
            Some(TokenType::Print) => self.print_statement(),
            Some(TokenType::LBrace) => {
                self.begin_scope();
                self.block()?;
                self.end_scope();
                Ok(())
            }
            Some(TokenType::If) => self.if_statement(),
            Some(TokenType::While) => self.while_statement(),
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

    fn block(&mut self) -> Result<()> {
        self.expect(TokenType::LBrace)?;

        loop {
            match self.peek() {
                Some(TokenType::RBrace) | None => break,
                _ => self.declaration()?,
            }
        }

        self.expect(TokenType::RBrace)?;
        Ok(())
    }

    #[inline]
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        loop {
            match self.locals.last() {
                Some(Local { depth, .. }) if depth > &self.scope_depth => {
                    self.emit_byte(OpCode::Pop as u8);
                    self.locals.pop();
                }
                _ => break,
            }
        }
    }

    fn if_statement(&mut self) -> Result<()> {
        self.expect(TokenType::If)?;

        self.expect(TokenType::LParen)?;
        self.expression()?;
        self.expect(TokenType::RParen)?;

        let then_jump = self.emit_jump(OpCode::JumpIfFalse as u8);

        self.emit_byte(OpCode::Pop as u8);
        self.statement()?;

        let else_jump = self.emit_jump(OpCode::Jump as u8);

        self.patch_jump(then_jump)?;
        self.emit_byte(OpCode::Pop as u8);

        if let Some(TokenType::Else) = self.peek() {
            self.advance()?;
            self.statement()?;
        }

        self.patch_jump(else_jump)
    }

    fn patch_jump(&mut self, offset: usize) -> Result<()> {
        let jump = self.chunk().code.len() - offset - 2;

        if jump > std::u16::MAX as usize {
            return Err(LoxError::CompileError);
        }

        self.chunk().code[offset] = ((jump as u16 >> 8) & 0xFF) as u8;
        self.chunk().code[offset + 1] = (jump as u16 & 0xFF) as u8;

        Ok(())
    }

    fn while_statement(&mut self) -> Result<()> {
        self.expect(TokenType::While)?;

        let loop_start = self.chunk().code.len();

        self.expect(TokenType::LParen)?;
        self.expression()?;
        self.expect(TokenType::RParen)?;

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse as u8);

        self.emit_byte(OpCode::Pop as u8);
        self.statement()?;

        self.emit_loop(loop_start)?;

        self.patch_jump(exit_jump)?;
        self.emit_byte(OpCode::Pop as u8);

        Ok(())
    }

    fn expr_statement(&mut self) -> Result<()> {
        dbg!("expr_statement");
        self.expression()?;
        self.expect(TokenType::Semicolon)?;
        self.emit_byte(OpCode::Pop as u8);
        Ok(())
    }

    pub fn expression(&mut self) -> Result<()> {
        dbg!("expression");
        self.parse_precedence(TokenType::Equal.precedence())
    }

    fn parse_precedence(&mut self, precedence: usize) -> Result<()> {
        dbg!("parse_precedence");
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
            Some(TokenType::Equal) if can_assign => {
                Err(LoxError::UnexpectedToken(Some(TokenType::Equal)))
            }
            _ => Ok(()),
        }
    }

    fn binary(&mut self) -> Result<()> {
        dbg!("binary");
        let op = self.advance()?.ok_or(LoxError::UnexpectedEOF)?;

        self.parse_precedence(op.precedence() + 1)?;

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
            token => return Err(LoxError::UnexpectedToken(Some(token))),
        }

        Ok(())
    }

    fn unary(&mut self) -> Result<()> {
        let op = self.advance()?.ok_or(LoxError::UnexpectedEOF)?;

        self.expression()?;

        match op {
            TokenType::Minus => self.emit_byte(OpCode::Negate as u8),
            TokenType::Bang => self.emit_byte(OpCode::Not as u8),
            token => return Err(LoxError::UnexpectedToken(Some(token))),
        }

        Ok(())
    }

    fn grouping(&mut self) -> Result<()> {
        self.expect(TokenType::LParen)?;

        self.expression()?;

        self.expect(TokenType::RParen).map(|_| ())
    }

    fn number(&mut self) -> Result<()> {
        dbg!("number");
        match self.advance()? {
            Some(TokenType::Num(n)) => {
                let handle = self.heap.insert(Value::Number(n));
                self.emit_const(handle)?;
                Ok(())
            }
            token => Err(LoxError::UnexpectedToken(token)),
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
            Some(TokenType::Str(value)) => {
                let handle = self.make_string(value)?;

                self.emit_const(handle)
            }
            token => Err(LoxError::UnexpectedToken(token)),
        }
    }

    fn variable(&mut self, can_assign: bool) -> Result<()> {
        dbg!("variable");
        let name = self.advance()?.ok_or(LoxError::UnexpectedEOF)?;
        self.named_variable(name, can_assign)
    }

    fn named_variable(&mut self, name: TokenType, can_assign: bool) -> Result<()> {
        dbg!("named_variable");
        match name {
            TokenType::Ident(value) => {
                // let arg = self.chunk().add_constant(Const::Str(s))?;
                let (arg, get_op, set_op) = match self.resolve_local(&value)? {
                    Some(idx) => (idx, OpCode::GetLocal, OpCode::SetLocal),
                    None => {
                        let handle = self.make_string(value)?;

                        (
                            self.chunk().add_constant(handle)?,
                            OpCode::GetGlobal,
                            OpCode::SetGlobal,
                        )
                    }
                };

                match self.peek() {
                    Some(TokenType::Equal) if can_assign => {
                        self.advance()?;
                        self.expression()?;
                        self.emit_bytes(set_op as u8, arg);
                    }
                    _ => self.emit_bytes(get_op as u8, arg),
                }

                Ok(())
            }
            token => Err(LoxError::UnexpectedToken(Some(token))),
        }
    }

    fn resolve_local(&mut self, name: &str) -> Result<Option<u8>> {
        for (idx, local) in self.locals.iter().rev().enumerate() {
            if &local.name == name {
                if local.depth == -1 {
                    return Err(LoxError::CompileError);
                }

                return Ok(Some(idx as u8));
            }
        }

        Ok(None)
    }

    fn and(&mut self) -> Result<()> {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse as u8);

        self.emit_byte(OpCode::Pop as u8);
        self.parse_precedence(TokenType::And.precedence())?;

        self.patch_jump(end_jump)
    }

    fn or(&mut self) -> Result<()> {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse as u8);
        let end_jump = self.emit_jump(OpCode::Jump as u8);

        self.patch_jump(else_jump)?;
        self.emit_byte(OpCode::Pop as u8);

        self.parse_precedence(TokenType::Or.precedence())?;
        self.patch_jump(end_jump)
    }

    fn call(&mut self) -> Result<()> {
        self.expect(TokenType::LParen)?;

        let arg_count = self.argument_list()?;

        self.emit_bytes(OpCode::Call as u8, arg_count);

        Ok(())
    }

    fn argument_list(&mut self) -> Result<u8> {
        let mut arg_count = 0;

        loop {
            match self.peek() {
                Some(TokenType::RParen) | None => break,
                _ => {
                    if arg_count == 255 {
                        return Err(LoxError::CompileError);
                    }

                    self.expression()?;

                    arg_count += 1;

                    match self.peek() {
                        Some(TokenType::RParen) | None => (),
                        _ => {
                            self.expect(TokenType::Comma)?;
                        }
                    };
                }
            }
        }

        self.expect(TokenType::RParen)?;
        Ok(arg_count)
    }

    fn prefix(&mut self, can_assign: bool) -> Result<()> {
        dbg!("prefix");
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
        dbg!("infix");
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
            TokenType::And => self.and(),
            TokenType::Or => self.or(),
            TokenType::LParen => self.call(),
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

    fn with_function_ctx<T>(&mut self, name: String, compile_fn: &mut T) -> Result<ObjFunction>
    where
        T: FnMut(&mut Self) -> Result<()>,
    {
        let handle = self.make_string(name)?;

        let old_scope_depth = mem::replace(&mut self.scope_depth, 0);

        let old_locals = mem::replace(
            &mut self.locals,
            vec![Local {
                depth: 0,
                name: String::from(""),
            }],
        );

        let old_function = mem::replace(
            &mut self.function,
            ObjFunction {
                arity: 0,
                chunk: Chunk::new(String::from("TODO: remove me")),
                name: Some(handle),
            },
        );

        compile_fn(self)?;

        self.scope_depth = old_scope_depth;
        self.locals = old_locals;

        self.emit_byte(OpCode::Return as u8);

        let compiled_function = mem::replace(&mut self.function, old_function);

        Ok(compiled_function)
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
            token => Err(LoxError::UnexpectedToken(token)),
        }
    }

    // #[inline]
    // pub fn current_fn_obj(&mut self) -> &mut ObjFunction {
    //     let i = self.functions.len() - 1;
    //     &mut self.functions[i]
    // }

    #[inline]
    pub fn chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    fn make_string(&mut self, value: String) -> Result<ValueHandle> {
        Ok(self
            .heap
            .insert(Value::Obj(LoxObj::Str(ObjString { value }))))
    }
}

impl<'a> Codegen for Compiler<'a> {
    fn emit_byte(&mut self, value: u8) {
        let line = self.line;
        self.chunk().write(value, line);
    }

    fn emit_const(&mut self, handle: ValueHandle) -> Result<()> {
        let const_idx = self.chunk().add_constant(handle)?;
        self.emit_bytes(OpCode::Constant as u8, const_idx);
        Ok(())
    }

    fn emit_jump(&mut self, value: u8) -> usize {
        self.emit_byte(value);
        self.emit_byte(0xFF);
        self.emit_byte(0xFF);

        self.chunk().code.len() - 2
    }

    fn emit_loop(&mut self, loop_start: usize) -> Result<()> {
        self.emit_byte(OpCode::Loop as u8);

        let offset = self.chunk().code.len() - loop_start + 2;

        if offset > std::u16::MAX as usize {
            return Err(LoxError::CompileError);
        }

        self.emit_byte(((offset >> 8) & 0xFF) as u8);
        self.emit_byte((offset & 0xFF) as u8);

        Ok(())
    }
}
