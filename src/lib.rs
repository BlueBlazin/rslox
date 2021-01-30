mod chunk;
mod codegen;
pub mod compiler;
mod debug;
mod error;
mod gc;
mod object;
mod opcodes;
mod scanner;
mod token;
mod value;
pub mod vm;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox() {
        use crate::codegen::Codegen;
        use crate::gc::Heap;
        use crate::opcodes::OpCode;
        // use crate::scanner::Scanner;

        let source = r#"
            var i = 0;
            while (i < 5) {
                print i;
                i = i + 1;
            }
        "#;

        let heap = Heap::new();

        let mut compiler = compiler::Compiler::new(source.chars(), heap);

        compiler.parse().unwrap();
        compiler.emit_byte(OpCode::Return as u8);

        println!("{:?}", compiler.chunk());

        let mut vm = vm::Vm::new(compiler.function, compiler.heap);
        vm.interpret().unwrap();

        // let mut scanner = Scanner::new(source.chars());
        // let tokens: Vec<_> = scanner.collect();
        // println!("{:?}", tokens);
    }
}
