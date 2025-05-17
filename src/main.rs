use parser::Parser;

mod lexer;
mod parser;
mod compiler;
mod typechecker;
mod shared;

fn main() {
    let mut p = Parser::new("5 + 4 * 0;\0");
    let prog = p.parse();
    dbg!(&prog.stmts[0]);
}
