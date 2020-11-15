#[derive(Debug, Clone)]
pub enum LoxError {
    CompileError,
    RuntimeError,
    StackOverflow,
    StackUnderflow,
    UnexpectedToken,
    UnexpectedEOF,
    TypeError,
    TooManyLocalVariables,
}

pub type Result<T> = std::result::Result<T, LoxError>;
