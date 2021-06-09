use crate::token::TokenType;
use crate::value::Value;

#[derive(Debug)]
pub enum LoxError {
    CompileError(&'static str),
    InternalCompilerError,
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
    UndefinedMethod(String),
    NonInstance,
    InvalidObject,
    InvalidField,
    InvalidClass,
    InvalidSuperClass,
    InvalidSubClass,
    InvalidSuper,
    InvalidUpvalue,
    InvalidArguments(&'static str),
    InternalVmError(&'static str),
    InvalidHandle,
}

#[derive(Debug)]
pub enum Internal {
    InvalidHandle,
    GlobalLookupFailure,
    CorruptedStack,
}

pub type Result<T> = std::result::Result<T, LoxError>;
