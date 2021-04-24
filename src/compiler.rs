use crate::chunk::Chunk;
use crate::codegen::Codegen;
use crate::error::{LoxError, Result};
use crate::gc::Heap;
use crate::object::{LoxObj, ObjClosure, ObjString};
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
    is_captured: bool,
}

#[derive(PartialEq)]
enum FunctionType {
    Function,
    Script,
}

#[derive(Clone, Debug)]
struct Upvalue {
    is_local: bool,
    index: u8,
}

enum UpvaluesKind {
    Current,
    Past(usize),
}

pub struct Compiler<'a> {
    scanner: Peekable<Scanner<'a>>,
    pub function: ObjClosure,
    fun_type: FunctionType,
    locals: Vec<Local>,
    scope_depth: isize,
    pub line: usize,
    pub heap: Heap<LoxObj>,
    upvalues: Vec<Upvalue>,
    locals_stack: Vec<Vec<Local>>,
    upvalues_stack: Vec<Vec<Upvalue>>,
}

impl<'a> Compiler<'a> {
    pub fn new(source: Chars<'a>, heap: Heap<LoxObj>) -> Self {
        // NOTE: I don't think we need to worry about GC here. Still, be mindful.
        let function = ObjClosure {
            arity: 0,
            chunk: Chunk::default(),
            name: None,
            upvalues: vec![],
            upvalue_count: 0,
            is_marked: false,
        };

        let mut locals = Vec::with_capacity(std::u8::MAX as usize + 1);

        locals.push(Local {
            name: String::from(""),
            depth: 0,
            is_captured: false,
        });

        Self {
            scanner: Scanner::new(source).peekable(),
            function,
            fun_type: FunctionType::Script,
            locals,
            scope_depth: 0,
            line: 0,
            heap,
            upvalues: Vec::with_capacity(u8::MAX as usize),
            locals_stack: vec![],
            upvalues_stack: vec![],
        }
    }

    pub fn compile(mut self) -> Result<()> {
        self.parse()
    }

    pub fn parse(&mut self) -> Result<()> {
        while self.peek().is_some() {
            self.declaration()?;
        }

        Ok(())
    }

    pub fn declaration(&mut self) -> Result<()> {
        dbg!("declaration");
        match self.peek() {
            Some(TokenType::Var) => self.var_declaration(),
            Some(TokenType::Fun) => self.fun_declaration(),
            _ => self.statement(),
        }
    }

    fn fun_declaration(&mut self) -> Result<()> {
        dbg!("fun_declaration");
        self.expect(TokenType::Fun)?;

        let (global, name) = self.parse_function_name()?;

        // The mark_initialized here is for functions defined outside
        // the global scope.
        self.mark_initialized();

        self.function(name)?;

        self.define_variable(global);

        Ok(())
    }

    fn function(&mut self, name: String) -> Result<()> {
        dbg!("function");
        let mut closure_obj = self.with_function_ctx(name, &mut |this| {
            this.begin_scope();

            this.parse_parameters()?;

            this.block()
        })?;

        closure_obj.upvalue_count = self.upvalues.len();

        let handle = self.heap.insert(LoxObj::Closure(closure_obj));
        let value = Value::Obj(handle);
        self.emit_closure(value)?;

        let upvalues = mem::replace(&mut self.upvalues, self.upvalues_stack.pop().unwrap());

        for Upvalue { is_local, index } in upvalues {
            self.emit_byte(is_local as u8);
            self.emit_byte(index);
        }

        Ok(())
    }

    fn parse_parameters(&mut self) -> Result<()> {
        dbg!("parse_parameters");
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

                    self.define_variable(param_const);

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
        dbg!("var_declaration");
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

        self.define_variable(const_idx);

        Ok(())
    }

    fn parse_function_name(&mut self) -> Result<(u8, String)> {
        match self.advance()? {
            Some(TokenType::Ident(id)) => {
                self.declare_variable(id.clone())?;

                if self.scope_depth > 0 {
                    Ok((0, id))
                } else {
                    let handle = self.make_string(id.clone());

                    let value = Value::Obj(handle);

                    Ok((self.chunk().add_constant(value)?, id))
                }
            }
            token => Err(LoxError::UnexpectedToken(token)),
        }
    }

    fn parse_variable(&mut self) -> Result<u8> {
        dbg!("parse_variable");
        match self.advance()? {
            Some(TokenType::Ident(id)) => {
                self.declare_variable(id.clone())?;

                if self.scope_depth > 0 {
                    Ok(0)
                } else {
                    let handle = self.make_string(id);

                    let value = Value::Obj(handle);

                    self.chunk().add_constant(value)
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

            if name == local.name {
                return Err(LoxError::CompileError);
            }
        }

        self.add_local(name)
    }

    fn add_local(&mut self, name: String) -> Result<()> {
        if self.locals.len() == 256 {
            return Err(LoxError::TooManyLocalVariables);
        }

        self.locals.push(Local {
            name,
            depth: -1,
            is_captured: false,
        });

        Ok(())
    }

    fn define_variable(&mut self, const_idx: u8) {
        dbg!("define_variable");
        if self.scope_depth <= 0 {
            self.emit_bytes(OpCode::DefineGlobal as u8, const_idx);
        } else {
            self.mark_initialized();
        }
    }

    fn mark_initialized(&mut self) {
        // Since mark_initialized is called indiscriminately
        // on function declaration, the conditional prevents
        // global functions from being marked.
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
            Some(TokenType::Return) => self.return_statement(),
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
        dbg!("block");
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
                Some(Local {
                    depth, is_captured, ..
                }) if depth > &self.scope_depth => {
                    if *is_captured {
                        self.emit_byte(OpCode::CloseUpvalue as u8);
                    } else {
                        self.emit_byte(OpCode::Pop as u8);
                    }

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

    fn return_statement(&mut self) -> Result<()> {
        if self.fun_type == FunctionType::Script {
            return Err(LoxError::CompileError);
        }

        self.expect(TokenType::Return)?;

        match self.peek() {
            Some(TokenType::Semicolon) => {
                self.emit_return();
            }
            _ => {
                self.expression()?;

                self.emit_byte(OpCode::Return as u8);
            }
        }

        self.expect(TokenType::Semicolon).map(|_| ())
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
        let op = self.advance()?.ok_or(LoxError::UnexpectedEof)?;

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
        let op = self.advance()?.ok_or(LoxError::UnexpectedEof)?;

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
                self.emit_const(Value::Number(n))?;

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
                let handle = self.make_string(value);

                let value = Value::Obj(handle);

                self.emit_const(value)
            }
            token => Err(LoxError::UnexpectedToken(token)),
        }
    }

    fn variable(&mut self, can_assign: bool) -> Result<()> {
        dbg!("variable");
        let name = self.advance()?.ok_or(LoxError::UnexpectedEof)?;
        self.named_variable(name, can_assign)
    }

    fn named_variable(&mut self, name: TokenType, can_assign: bool) -> Result<()> {
        dbg!("named_variable");
        match name {
            TokenType::Ident(value) => {
                let arg;
                let get_op;
                let set_op;

                if let Some(idx) = self.resolve_local(&value)? {
                    arg = idx;
                    get_op = OpCode::GetLocal;
                    set_op = OpCode::SetLocal;
                } else if let Some(idx) = self.resolve_upvalue(&value)? {
                    arg = idx;
                    get_op = OpCode::GetUpvalue;
                    set_op = OpCode::SetUpvalue;
                } else {
                    let handle = self.make_string(value);

                    let value = Value::Obj(handle);

                    arg = self.chunk().add_constant(value)?;
                    get_op = OpCode::GetGlobal;
                    set_op = OpCode::SetGlobal;
                }

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
        self.resolve_local_with(name, &self.locals)
    }

    fn resolve_local_with(&self, name: &str, locals: &[Local]) -> Result<Option<u8>> {
        for (idx, local) in locals.iter().enumerate().rev() {
            if local.name == name {
                if local.depth == -1 {
                    return Err(LoxError::CompileError);
                }

                return Ok(Some(idx as u8));
            }
        }

        Ok(None)
    }

    fn add_upvalue(
        &mut self,
        upvalues_kind: UpvaluesKind,
        index: u8,
        is_local: bool,
    ) -> Result<u8> {
        let upvalues = match upvalues_kind {
            UpvaluesKind::Current => &mut self.upvalues,
            UpvaluesKind::Past(i) => &mut self.upvalues_stack[i],
        };

        for upvalue in upvalues.iter() {
            if (upvalue.index == index) && (upvalue.is_local == is_local) {
                return Ok(index);
            }
        }

        if upvalues.len() >= u8::MAX as usize {
            return Err(LoxError::_TempDevError("too many upvalues"));
        }

        upvalues.push(Upvalue { index, is_local });

        Ok(upvalues.len() as u8 - 1)
    }

    // We implement a poor man's recursion with an explicit pointer and loop.
    fn resolve_upvalue(&mut self, name: &str) -> Result<Option<u8>> {
        if self.locals_stack.is_empty() {
            return Ok(None);
        }

        let mut i = self.locals_stack.len() - 1;

        let mut upvalues_kind = UpvaluesKind::Current;

        loop {
            // If we find the local in some functions enclosing locals array, then it adds a local.
            // The rest will add upvalues all the way to the top as we unwind.
            if let Some(idx) = self.resolve_local_with(name, &self.locals_stack[i])? {
                // add local
                let mut index = self.add_upvalue(upvalues_kind, idx, true)?;

                // mark local
                self.locals_stack[i][idx as usize].is_captured = true;

                // unwind
                while i < self.locals_stack.len() - 1 {
                    // upvalues = &mut self.upvalues_stack[i];
                    upvalues_kind = UpvaluesKind::Past(i as usize);
                    index = self.add_upvalue(upvalues_kind, index, false)?;
                    i += 1;
                }

                return Ok(Some(index));
            }

            // If we reach the bottom and don't find a matching local that means
            // it doesn't exist. So we just return None right away.
            if i == 0 {
                return Ok(None);
            }

            upvalues_kind = UpvaluesKind::Past(i as usize);
            i -= 1;
        }
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
        match self.peek().ok_or(LoxError::UnexpectedEof)? {
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
        match self.peek().ok_or(LoxError::UnexpectedEof)? {
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

    fn with_function_ctx<T>(&mut self, name: String, compile_fn: &mut T) -> Result<ObjClosure>
    where
        T: FnMut(&mut Self) -> Result<()>,
    {
        let handle = self.make_string(name);

        let old_scope_depth = mem::replace(&mut self.scope_depth, 0);

        let old_fun_type = mem::replace(&mut self.fun_type, FunctionType::Function);

        let old_function = mem::replace(
            &mut self.function,
            ObjClosure {
                arity: 0,
                chunk: Chunk::default(),
                name: Some(handle),
                upvalues: vec![],
                upvalue_count: 0,
                is_marked: false,
            },
        );

        self.locals_stack.push(mem::replace(
            &mut self.locals,
            vec![Local {
                depth: 0,
                name: String::from(""),
                is_captured: false,
            }],
        ));

        self.upvalues_stack
            .push(mem::replace(&mut self.upvalues, vec![]));

        compile_fn(self)?;

        self.scope_depth = old_scope_depth;
        self.locals = self.locals_stack.pop().unwrap();
        self.fun_type = old_fun_type;

        self.emit_return();

        Ok(mem::replace(&mut self.function, old_function))
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

    #[inline]
    pub fn chunk(&mut self) -> &mut Chunk {
        &mut self.function.chunk
    }

    fn make_string(&mut self, value: String) -> ValueHandle {
        self.heap.insert(LoxObj::Str(ObjString {
            value,
            is_marked: false,
        }))
    }

    fn emit_return(&mut self) {
        self.emit_byte(OpCode::Nil as u8);
        self.emit_byte(OpCode::Return as u8);
    }
}

impl<'a> Codegen for Compiler<'a> {
    #[inline]
    fn emit_byte(&mut self, value: u8) {
        let line = self.line;
        self.chunk().write(value, line);
    }

    fn emit_const(&mut self, value: Value) -> Result<()> {
        let const_idx = self.chunk().add_constant(value)?;
        self.emit_bytes(OpCode::Constant as u8, const_idx);
        Ok(())
    }

    fn emit_closure(&mut self, value: Value) -> Result<()> {
        dbg!("emit_closure");
        let const_idx = self.chunk().add_constant(value)?;
        self.emit_bytes(OpCode::Closure as u8, const_idx);
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
