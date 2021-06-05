use crate::token::TokenType;
use crate::value::Value;

#[derive(Debug)]
pub enum LoxError {
    CompileError,
    RuntimeError,
    StackOverflow,
    StackUnderflow,
    UnexpectedToken(Option<TokenType>),
    UnexpectedEof,
    TypeError,
    TooManyLocalVariables,
    UnexpectedCharacter,
    InvalidTypeForAddition,
    InternalError(Internal),
    InvalidTypeForEquals,
    ValueNotCallable,
    UnexpectedValue(Value),
    UndefinedProperty(String),
    _TempDevError(&'static str),
}

#[derive(Debug)]
pub enum Internal {
    InvalidHandle,
    GlobalLookupFailure,
    CorruptedStack,
}

pub type Result<T> = std::result::Result<T, LoxError>;
