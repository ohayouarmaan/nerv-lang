use core::panic;
use std::{collections::HashMap, fs::File};
use crate::shared::{
    compiler_defaults::SIZES, errors::CompilerError, meta::{ AnyMetadata, NumberType }, parser_nodes::{Expression, ExpressionStatement, Program, Statement, VarDeclarationStatement}, tokens::TokenType
};

pub struct Symbol {
    pub offset: isize,
    pub size: usize
}

#[allow(dead_code)]
pub struct Compiler<'a> {
    pub prog: Program<'a>,
    pub file_handler: File,
    pub asm: Vec<String>,
    pub symbol_table: HashMap<&'a str, Symbol>,
    pub current_stack_offset: isize,
    pub data_section: Vec<String>,
    pub data_counter: usize
}

#[allow(dead_code)]
impl<'a> Compiler<'a> {
    pub fn new(ast: Program<'a>, out_file: &'a str) -> Result<Self, CompilerError> {
        dbg!(&ast);
        let file = match File::create(out_file) {
            Ok(handler) => handler,
            Err(_) => return Err(CompilerError::IllegalOutputFile)
        };
        let asm = vec!["global main\n".to_string()];
        let data_section = vec!["section .data\n".to_string()];
        Ok(Self {
            prog: ast,
            file_handler: file,
            asm,
            data_section,
            symbol_table: HashMap::new(),
            current_stack_offset: 0,
            data_counter: 0
        })
    }

    pub fn compile(&mut self) -> Result<(), CompilerError> {
        let data_index = self.asm.len();
        self.asm.push("section .text\n".to_string());
        self.asm.push("main:\n".to_string());
        self.asm.push("\tpush rbp\n".to_string());
        self.asm.push("\tmov rbp, rsp\n".to_string());
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

        for i in 0..self.data_section.len() {
            self.asm.insert(data_index + i, self.data_section[i].clone());
        }

        self.asm.push("\tmov rax, 60\n".to_string());
        self.asm.push("\tmov rdi, 0\n".to_string());
        self.asm.push("\tsyscall\n".to_string());
        Ok(())
    }

    pub fn compile_statement(&mut self, stmt: &Statement<'a>) -> Result<Vec<String>, CompilerError> {
        match stmt {
            Statement::ExpressionStatement(e) => self.compile_expression_statement(e),
            Statement::VarDeclaration(var) => self.compile_variable_declaration_statement(var),
        }
    }

    pub fn compile_variable_declaration_statement(&mut self, stmt: &VarDeclarationStatement<'a>) -> Result<Vec<String>, CompilerError> {
        let mut asms_main = vec![];
        asms_main.extend(self.compile_expression(&stmt.value)?);
        match stmt.variable_type {
            TokenType::DInteger => {
                self.asm.push("\tsub rsp, 4\n".to_string());
                self.current_stack_offset -= SIZES.d_int as isize;
                self.symbol_table.insert(stmt.name, Symbol {
                    offset: self.current_stack_offset,
                    size: SIZES.d_int
                });
                asms_main.push("\tmov DWORD [rsp], eax\n".to_string());
            },
            TokenType::DFloat => {
                self.asm.push("\tsub rsp, 4\n".to_string());
                self.current_stack_offset -= SIZES.d_float as isize;
                self.symbol_table.insert(stmt.name, Symbol {
                    offset: self.current_stack_offset,
                    size: SIZES.d_int
                });
                asms_main.push("\tmovq QWORD [rsp], rax\n".to_string());
            },
            TokenType::DString => {
                self.asm.push("\tsub rsp, 8\n".to_string());
                self.current_stack_offset -= SIZES.d_ptr as isize;
                self.symbol_table.insert(stmt.name, Symbol {
                    offset: self.current_stack_offset,
                    size: SIZES.d_ptr
                });
                asms_main.push("\tmov QWORD [rsp], rax\n".to_string());
            },
            _ => {
                return Err(CompilerError::UnknownDataType);
            }
        }
        Ok(asms_main)
    }

    pub fn compile_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) -> Result<Vec<String>, CompilerError> {
        self.compile_expression(&stmt.value)
    }

    // #[allow(clippy::only_used_in_recursion)]
    pub fn compile_expression(&mut self, expr: &Expression<'a>) -> Result<Vec<String>, CompilerError> {
        let mut asms_main = vec![];
        match expr {
            Expression::Binary(bin) => {
                if let Ok(compiled_asms) = self.compile_expression(&bin.left) {
                    for asm in compiled_asms {
                        asms_main.push(asm);
                    }
                }

                self.asm.push("\tsub rsp, 8\n".to_string());
                asms_main.push("\tpush rax\n".to_string());

                if let Ok(compiled_asms) = self.compile_expression(&bin.right) {
                    for asm in compiled_asms {
                        asms_main.push(asm);
                    }
                }

                asms_main.push("\tpop rbx\n".to_string());
                asms_main.push("\tadd rsp, 8\n".to_string());
                match bin.operator.token_type {
                    TokenType::Plus => asms_main.push("\tadd rax, rbx\n".to_string()),
                    TokenType::Minus =>asms_main.push("\tsub rax, rbx\n".to_string()),
                    TokenType::Star => asms_main.push("\timul rax, rbx\n".to_string()),
                    TokenType::Slash => {
                        asms_main.push("\tmov rdx, 0\n".to_string());
                        asms_main.push("\tmov rcx, rax\n".to_string());
                        asms_main.push("\tmov rax, rbx\n".to_string());
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
                AnyMetadata::Identifier { value } => {
                    let variable_symbol = self.symbol_table.get(value);
                    match variable_symbol {
                        Some(s) => {
                            // Choose register based on variable size
                            match s.size {
                                4 => {
                                    asms_main.push(format!("\tmov eax, DWORD [rbp{}]\n", s.offset));
                                }
                                8 => {
                                    asms_main.push(format!("\tmov rax, QWORD [rbp{}]\n", s.offset));
                                }
                                2 => {
                                    asms_main.push(format!("\tmov ax, WORD [rbp{}]\n", s.offset));
                                    asms_main.push("\tmovzx rax, ax\n".to_string()); // Zero-extend to 64-bit
                                }
                                1 => {
                                    asms_main.push(format!("\tmov al, BYTE [rbp{}]\n", s.offset));
                                    asms_main.push("\tmovzx rax, al\n".to_string()); // Zero-extend to 64-bit
                                }
                                _ => {
                                    return Err(CompilerError::UnknownDataType);
                                }
                            }
                        }
                        None => {
                            panic!("Unknown variable {:?}", *value);
                        }
                    }
                }
                AnyMetadata::String { value } => {
                    self.data_section.push(format!("var_{} db {}, 0\n", self.data_counter, (*value)).to_string());
                    self.data_section.push(format!("var_len_{} equ {}\n", self.data_counter, (*value).len()).to_string());
                    asms_main.push(format!("\tlea rax, [rel var_{}]\n", self.data_counter).to_string());
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

