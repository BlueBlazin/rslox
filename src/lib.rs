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
        use crate::gc::Heap;

        let source = r#"
        var globalSet;
        var globalGet;
        
        fun main() {
            var a = "initial";
        
            fun set() { a = "updated"; }
            fun get() { print a; }
        
            globalSet = set;
            globalGet = get;
        }

        main();
        globalSet();
        globalGet();
        "#;

        let heap = Heap::new();

        let mut compiler = compiler::Compiler::new(source.chars(), heap);

        compiler.parse().unwrap();

        println!("{:?}", compiler.chunk());

        let mut vm = vm::Vm::new(compiler.heap);
        vm.interpret(compiler.function).unwrap();
    }
}
