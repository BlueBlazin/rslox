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
            class Oops {
                init() {
                    fun f() {
                        print "not a method";
                    }
                
                    this.field = f;
                }
            }
            
            var oops = Oops();
            oops.field();
        "#;

        let heap = Heap::default();

        let mut compiler = compiler::Compiler::new(source.chars(), heap);

        compiler.compile().unwrap();

        // println!("{:?}", compiler.chunk());
        println!("End of compilation\n");

        let mut vm = vm::Vm::new(compiler.heap);
        vm.interpret(compiler.function).unwrap();
    }
}
