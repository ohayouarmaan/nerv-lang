use parser::Parser;
use typechecker::TypeChecker;
use std::env;
use std::fs;
use std::io::Write;

mod lexer;
mod parser;
mod compiler;
mod typechecker;
mod shared;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_file>", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let source_code = fs::read_to_string(input_path)
        .expect("Error while reading the input file.");

    let mut parser = Parser::new(&source_code);
    let program = parser.parse();

    let mut type_checker = TypeChecker::new(program.clone());
    type_checker.check();

    let mut compiler = match compiler::Compiler::new(program, output_path) {
        Ok(c) => c,
        Err(e) => panic!("{:?}", e),
    };

    let _ = compiler.compile();

    for asm in compiler.asm {
        let _ = compiler.file_handler.write_all(asm.as_bytes());
    }
}
