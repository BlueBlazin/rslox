mod chunk;
mod codegen;
mod compiler;
mod debug;
mod error;
mod opcodes;
mod scanner;
mod token;
mod value;
mod vm;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox() {
        let mut compiler = compiler::Compiler::new("true".chars());
        let mut vm = vm::Vm::new();

        // vm.interpret(chunk);
    }
}
