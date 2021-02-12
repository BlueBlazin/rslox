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

        let source = r#"
        fun foo() {
            print "bar";
        }

        foo();
        "#;

        let heap = Heap::new();

        let mut compiler = compiler::Compiler::new(source.chars(), heap);

        compiler.parse().unwrap();
        compiler.emit_byte(OpCode::Return as u8);

        println!("{:?}", compiler.chunk());

        let mut vm = vm::Vm::new(compiler.heap);
        vm.interpret(compiler.function).unwrap();

        // println!(
        //     "{:?}",
        //     &vm.heap
        //         .objects
        //         .iter()
        //         .map(|x| vm.heap.get(x))
        //         .collect::<Vec<_>>()
        // );
    }
}
