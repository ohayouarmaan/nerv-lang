use parser::Parser;
use typechecker::TypeChecker;
use std::io::Write;
use std::fs;
mod lexer;
mod parser;
mod compiler;
mod typechecker;
mod shared;

fn main() {
    let source_code = fs::read_to_string("./examples/test.nerv").expect("Error while reading the file.");
    let mut p = Parser::new(&source_code);
    let prog = p.parse();
    let mut x = TypeChecker::new(prog.clone());
    x.check();
    let mut c = match compiler::Compiler::new(prog, "out.s") {
        Ok(t) => t,
        Err(e) => panic!("{:?}", e)
    };
    let _ = c.compile();
    for asm in c.asm{
        let _ = c.file_handler.write_all(asm.as_bytes());
    }
}
