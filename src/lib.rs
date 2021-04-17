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
        var globalOne;
        var globalTwo;

        fun main() {
            {
                var a = "one";
                fun one() {
                    print a;
                }
                globalOne = one;
            }

            {
                var a = "two";
                fun two() {
                    print a;
                }
                globalTwo = two;
            }
        }

        main();
        globalOne();
        globalTwo();
        "#;

        let heap = Heap::default();

        let mut compiler = compiler::Compiler::new(source.chars(), heap);

        compiler.parse().unwrap();

        println!("{:?}", compiler.chunk());

        let mut vm = vm::Vm::new(compiler.heap);
        vm.interpret(compiler.function).unwrap();
    }
}
