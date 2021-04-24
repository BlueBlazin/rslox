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
        fun fib(n) {
            if (n < 2) {
                return n;
            }

            return fib(n - 1) + fib(n - 2);
        }

        var x = "Hello, world!";
        var y = "Assertial failed";

        var z = fib(8 * 2);

        if (z > 1000) {
            print y;
        } else {
            print x + " " + "and my GC!";
            print z;
        }
        "#;

        let heap = Heap::default();

        let mut compiler = compiler::Compiler::new(source.chars(), heap);

        compiler.parse().unwrap();

        // println!("{:?}", compiler.chunk());
        println!("End of compilation\n");

        let mut vm = vm::Vm::new(compiler.heap);
        vm.interpret(compiler.function).unwrap();
    }
}
