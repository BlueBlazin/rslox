#[derive(Debug)]
pub enum LoxError {
    CompileError,
    RuntimeError,
    StackOverflow,
    StackUnderflow,
    UnexpectedToken,
    UnexpectedEOF,
}

pub type Result<T> = std::result::Result<T, LoxError>;
