use parser::Parser;
use std::io::Write;
mod lexer;
mod parser;
mod compiler;
mod typechecker;
mod shared;

fn main() {
    let mut p = Parser::new("dec fifteen int = 5 * 3;\0");
    let prog = p.parse();
    let mut c = match compiler::Compiler::new(prog, "test.s") {
        Ok(t) => t,
        Err(e) => panic!("{:?}", e)
    };
    let _ = c.compile();
    for asm in c.asm{
        let _ = c.file_handler.write_all(asm.as_bytes());
    }
}
