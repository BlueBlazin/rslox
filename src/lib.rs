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
        use crate::codegen::Codegen;
        use crate::opcodes::OpCode;
        use crate::scanner::Scanner;

        let source = r#"
            var i = 0;
            while (i < 5) {
                print i;
                i = i + 1;
            }
        "#;

        let mut compiler = compiler::Compiler::new(source.chars());
        let mut vm = vm::Vm::new();

        compiler.parse().unwrap();
        compiler.emit_byte(OpCode::Return as u8);

        println!("{:?}", compiler.chunk);

        vm.interpret(compiler.chunk).unwrap();

        // let mut scanner = Scanner::new(source.chars());
        // let tokens: Vec<_> = scanner.collect();
        // println!("{:?}", tokens);
    }
}
