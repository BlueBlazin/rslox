#[derive(Debug)]
pub enum LoxError {
    CompileError,
    RuntimeError,
    StackOverflow,
    StackUnderflow,
}

pub type Result<T> = std::result::Result<T, LoxError>;
