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
        let mut compiler = compiler::Compiler::new("1 + 1".chars());
        let mut vm = vm::Vm::new();

        compiler.expression().unwrap();

        vm.interpret(compiler.chunk).unwrap();

        println!("{:?}", vm.stack);
    }
}
