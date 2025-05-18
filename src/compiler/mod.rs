use std::fs::File;
use crate::shared::{
    errors::CompilerError,
    meta::{ AnyMetadata, NumberType },
    parser_nodes::{Expression, ExpressionStatement, Program, Statement}, tokens::TokenType
};

#[allow(dead_code)]
pub struct Compiler<'a> {
    pub prog: Program<'a>,
    pub file_handler: File,
    pub asm: Vec<String>
}

#[allow(dead_code)]
impl<'a> Compiler<'a> {
    pub fn new(ast: Program<'a>, out_file: &'a str) -> Result<Self, CompilerError> {
        let file = match File::create(out_file) {
            Ok(handler) => handler,
            Err(_) => return Err(CompilerError::IllegalOutputFile)
        };
        let asm = vec!["global _start\n".to_string()];
        Ok(Self {
            prog: ast,
            file_handler: file,
            asm
        })
    }

    pub fn compile(&mut self) -> Result<(), CompilerError> {
        self.asm.push("_start:\n".to_string());
        for statement in self.prog.stmts.clone() {
            match self.compile_statement(&statement) {
                Err(_) => {

                }
                Ok(asm) => {
                    for a in asm {
                        self.asm.push(a);
                    }
                }
            }
        }
        self.asm.push("\tmov rax, 60\n".to_string());
        self.asm.push("\tmov rdi, 0\n".to_string());
        self.asm.push("\tsyscall\n".to_string());
        Ok(())
    }

    pub fn compile_statement(&mut self, stmt: &Statement<'a>) -> Result<Vec<String>, CompilerError> {
        match stmt {
            Statement::ExpressionStatement(e) => self.compile_expression_statement(e),
            Statement::VarDeclaration(_) => todo!(),
        }
    }

    pub fn compile_expression_statement(&self, stmt: &ExpressionStatement) -> Result<Vec<String>, CompilerError> {
        self.compile_expression(&stmt.value)
    }

    #[allow(clippy::only_used_in_recursion)] // we will use &self more in future, for now to get
    // rid of weird linter issues we allow it.
    pub fn compile_expression(&self, expr: &Expression<'a>) -> Result<Vec<String>, CompilerError> {
        let mut asms_main = vec![];
        match expr {
            Expression::Binary(bin) => {
                if let Ok(compiled_asms) = self.compile_expression(&bin.left) {
                    for asm in compiled_asms {
                        asms_main.push(asm);
                    }
                }

                asms_main.push("\tpush rax\n".to_string());

                if let Ok(compiled_asms) = self.compile_expression(&bin.right) {
                    for asm in compiled_asms {
                        asms_main.push(asm);
                    }
                }

                asms_main.push("\tpop rbx\n".to_string());
                match bin.operator.token_type {
                    TokenType::Plus => asms_main.push("\tadd rax, rbx\n".to_string()),
                    TokenType::Minus =>asms_main.push("\tsub rax, rbx\n".to_string()),
                    TokenType::Star => asms_main.push("\timul rax, rbx\n".to_string()),
                    TokenType::Slash => {
                        asms_main.push("\tmov rdx, 0\n".to_string());
                        asms_main.push("\tmov rcx, rax\n".to_string()); // right in rcx
                        asms_main.push("\tmov rax, rbx\n".to_string()); // left in rax
                        asms_main.push("\tdiv rcx\n".to_string());
                    }
                    _ => unimplemented!("Operator not implemented"),
                }
            }

            Expression::Literal(lit) => match &lit.value.meta_data {
                AnyMetadata::Number { value: NumberType::Integer(val) } => {
                    asms_main.push(format!("\tmov rax, {}\n", val));
                }
                AnyMetadata::Number { value: NumberType::Float(val) } => {
                    asms_main.push(format!("\tmov rax, {:.9}\n", val));
                }
                _ => unimplemented!("Only number literals supported for now"),
            },
            
            _ => unimplemented!("Only number literals supported for now"),
        }
        Ok(asms_main)
    }
}

#[cfg(test)]
mod tests {
    // use std::io::Write;
    // use crate::parser::Parser;
    // use super::*;

    // #[test]
    // fn check_simple_arithmetic() {
    //     let mut p = Parser::new("5 + 4 * 3\0");
    //     let program = p.parse();
    //     let mut c = match Compiler::new(program, "test.s") {
    //         Ok(t) => t,
    //         Err(e) => panic!("{:?}", e)
    //     };
    //     let _ = c.compile();
    //     dbg!(&c.asm);
    //     let correct = vec![
    //         "global _start\n",
    //         "_start:\n",
    //         "\tmov rax, 5\n",
    //         "\tpush rax\n",
    //         "\tmov rax, 4\n",
    //         "\tpush rax\n",
    //         "\tmov rax, 3\n",
    //         "\tpop rbx\n",
    //         "\timul rax, rbx\n",
    //         "\tpop rbx\n",
    //         "\tadd rax, rbx\n",
    //         "\tmov rax, 60\n",
    //         "\tmov rdi, 0\n",
    //         "\tsyscall\n",
    //     ];
    //     assert_eq!(correct, c.asm);
    //     for asm in c.asm{
    //         let _ = c.file_handler.write_all(asm.as_bytes());
    //     }
    // }
}

