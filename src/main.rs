use rslox::interpret;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let filepath = &args[1];
    println!("{}", filepath);

    let source = fs::read_to_string(filepath).unwrap();

    interpret(source).unwrap();
}
