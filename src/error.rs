use crate::token::TokenType;

#[derive(Debug)]
pub enum LoxError {
    CompileError,
    RuntimeError,
    StackOverflow,
    StackUnderflow,
    UnexpectedToken(Option<TokenType>),
    UnexpectedEOF,
    TypeError,
    TooManyLocalVariables,
    UnexpectedCharacter,
}

pub type Result<T> = std::result::Result<T, LoxError>;
