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

pub fn interpret(source: String) -> Result<(), error::LoxError> {
    let heap = gc::Heap::default();

    let mut compiler = compiler::Compiler::new(source.chars(), heap);

    compiler.compile()?;

    let mut vm = vm::Vm::new(compiler.heap);

    vm.interpret(Box::from(compiler.function))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox() {
        use crate::gc::Heap;

        let source = r#"
            class Doughnut {
                finish(number) {
                    print number;
                }
            }

            class Cruller < Doughnut {
                finish(number) {
                    super.finish(number);
                }
            }

            var cruller = Cruller();
            cruller.finish(42);
        "#;

        let heap = Heap::default();

        let mut compiler = compiler::Compiler::new(source.chars(), heap);

        compiler.compile().unwrap();

        // println!("{:?}", compiler.chunk());
        println!("End of compilation\n");

        let mut vm = vm::Vm::new(compiler.heap);
        vm.interpret(Box::from(compiler.function)).unwrap();
    }
}
