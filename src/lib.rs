mod chunk;
mod codegen;
mod compiler;
mod debug;
mod error;
mod object;
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
        use codegen::Codegen;

        let mut compiler = compiler::Compiler::new("100 * (2 + 1)".chars());
        let mut vm = vm::Vm::new();

        compiler.expression().unwrap();
        compiler.emit_byte(opcodes::OpCode::Return as u8);

        println!("{:?}", compiler.chunk);

        vm.interpret(compiler.chunk).unwrap();
    }
}
