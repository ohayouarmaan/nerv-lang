use core::panic;
use std::{collections::HashMap, fs::File};
use crate::{main, shared::{
    compiler_defaults::SIZES, errors::CompilerError, meta::{ AnyMetadata, NumberType }, parser_nodes::{Expression, ExpressionStatement, FunctionDeclaration, Program, ReturnStatement, Statement, VarDeclarationStatement}, tokens::TokenType
}};

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
    pub text_section: Vec<String>,
    pub data_counter: usize,
    pub label_table: HashMap<String, Vec<String>>,
}

#[allow(dead_code)]
impl<'a> Compiler<'a> {
    pub fn new(ast: Program<'a>, out_file: &'a str) -> Result<Self, CompilerError> {
        dbg!(&ast);
        let file = match File::create(out_file) {
            Ok(handler) => handler,
            Err(_) => return Err(CompilerError::IllegalOutputFile)
        };
        let asm = vec![];
        let data_section = vec!["section .data\n".to_string()];
        let text_section = vec!["section .text\n".to_string()];
        Ok(Self {
            prog: ast,
            file_handler: file,
            asm,
            data_section,
            text_section,
            symbol_table: HashMap::new(),
            current_stack_offset: 0,
            data_counter: 0,
            label_table: HashMap::new()
        })
    }

    pub fn compile(&mut self) -> Result<(), CompilerError> {
        let progs = self.prog.stmts.clone();
        for statement in progs {
            if let Statement::FunctionDeclaration(fx) = &statement {
                let compiled_fx = self.compile_function_declaration_statement(fx);
                if let Ok((function_name, body)) = compiled_fx {
                    self.text_section.push(format!("\tglobal {}\n", function_name).to_string());
                    self.label_table.insert(function_name, body);
                } else {
                    println!("Err: {:?}", compiled_fx);
                }
            }
        }
        self.asm.extend_from_slice(&self.data_section);
        self.asm.extend_from_slice(&self.text_section);

        for t in self.label_table.iter() {
            self.asm.push(format!("{}:\n", t.0));
            for stmt in t.1 {
                self.asm.push(stmt.to_string());
            }
        }

        Ok(())
    }

    pub fn compile_statement(&mut self, stmt: &Statement<'a>) -> Result<Vec<String>, CompilerError> {
        match stmt {
            Statement::ExpressionStatement(e) => self.compile_expression_statement(e),
            Statement::VarDeclaration(var) => self.compile_variable_declaration_statement(var),
            Statement::ReturnStatement(ret) => self.compile_return_statement(ret),
            _ => {
                Err(CompilerError::UnexpectedStandaloneBlock)
            }
        }
    }

    pub fn compile_variable_declaration_statement(&mut self, stmt: &VarDeclarationStatement<'a>) -> Result<Vec<String>, CompilerError> {
        let mut asms_main = vec!["\n\t; VARIABLE DECLARATION\n".to_string()];
        asms_main.extend(self.compile_expression(&stmt.value, "rax")?);
        match stmt.variable_type {
            TokenType::DInteger => {
                asms_main.push("\tsub rsp, 4\n".to_string());
                self.current_stack_offset -= SIZES.d_int as isize;
                self.symbol_table.insert(stmt.name, Symbol {
                    offset: self.current_stack_offset,
                    size: SIZES.d_int
                });
                asms_main.push("\tmov DWORD [rsp], eax\n".to_string());
            },
            TokenType::DFloat => {
                asms_main.push("\tsub rsp, 4\n".to_string());
                self.current_stack_offset -= SIZES.d_float as isize;
                self.symbol_table.insert(stmt.name, Symbol {
                    offset: self.current_stack_offset,
                    size: SIZES.d_int
                });
                asms_main.push("\tmovq QWORD [rsp], rax\n".to_string());
            },
            TokenType::DString => {
                asms_main.push("\tsub rsp, 8\n".to_string());
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
        asms_main.push("\n".to_string());
        Ok(asms_main)
    }

    pub fn compile_function_declaration_statement(&mut self, stmt: &FunctionDeclaration<'a>) -> Result<(String, Vec<String>), CompilerError> {
        let old_sp = self.current_stack_offset;
        self.current_stack_offset = 0;
        let mut body_stmts = vec![
            "\tpush rbp\n".to_string(),
            "\tmov rbp, rsp\n".to_string(),
        ];
        if stmt.arity > 0 {
            let order = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
            let order_32_bit = ["edi", "esi", "edx", "ecx", "e8", "e9"];
            (0..stmt.arity).for_each(|i| {
                let (size, operand_size, register): (usize, &'a str, &'a str) = match stmt.arguments[i].arg_type {
                    TokenType::DInteger => (4, "DWORD", order_32_bit[i]),
                    TokenType::DFloat => (8, "QWORD", order[i]),
                    TokenType::DString => (8, "QWORD", order[i]),
                    _ => unimplemented!("Can not determine size of other things.")
                };
                body_stmts.push(format!("\tsub rsp, {}\n", size));
                self.current_stack_offset -= size as isize;
                body_stmts.push(format!("\tmov {} [rbp-{}], {}\n", operand_size, self.current_stack_offset.abs(), register));
                self.symbol_table.insert(stmt.arguments[i].name, Symbol {
                    offset: self.current_stack_offset,
                    size
                });
            });
        }
        // will be used to later check if the function is returning by itself or not.
        let mut has_explicit_return = false;
        for body_statement in &stmt.body.values {
            if let Statement::ReturnStatement(_) = body_statement {
                has_explicit_return = true;
            }
            if let Ok(value) = self.compile_statement(body_statement) {
                for compiled_stmt in value {
                    body_stmts.push(compiled_stmt);
                }
            }
        }

        if !has_explicit_return {
            body_stmts.push("\tmov rax, 0\n".to_string());
            body_stmts.push("\tleave\n".to_string());
            body_stmts.push("\tret\n".to_string());
        } else {
            body_stmts.push("\tleave\n".to_string());
            body_stmts.push("\tret\n".to_string());
        }
        self.current_stack_offset = old_sp;
        Ok((stmt.name.to_string(), body_stmts))
    }

    pub fn compile_return_statement(&mut self, ret: &ReturnStatement<'a>) -> Result<Vec<String>, CompilerError> {
        let x = &ret.value;
        let mut main_asm_for_return = vec![];
        if let Ok(compiled_literal) = self.compile_expression(x, "rax") {
            for v in compiled_literal {
                main_asm_for_return.push(v);
            }
            Ok(main_asm_for_return)
        } else {
            Ok(vec![
                "mov rax, -1".to_string(),
                "leave".to_string(),
                "ret".to_string()
            ])
        }
    }

    pub fn compile_expression_statement(&mut self, stmt: &ExpressionStatement<'a>) -> Result<Vec<String>, CompilerError> {
        let compiled = self.compile_expression(&stmt.value, "rax");
        if let Ok(mut x) = compiled{
            x.insert(0, "; Expression Statement".to_string());
            Ok(x)
        } else {
            compiled
        }
    }

    // #[allow(clippy::only_used_in_recursion)]
    pub fn compile_expression(&mut self, expr: &Expression<'a>, register: &'a str) -> Result<Vec<String>, CompilerError> {
        let mut asms_main = vec![];
        match expr {
            Expression::Binary(bin) => {
                if let Ok(compiled_asms) = self.compile_expression(&bin.left, register) {
                    for asm in compiled_asms {
                        asms_main.push(asm);
                    }
                }

                asms_main.push("\tsub rsp, 8\n".to_string());
                asms_main.push(format!("\tpush {}\n", register));

                if let Ok(compiled_asms) = self.compile_expression(&bin.right, register) {
                    for asm in compiled_asms {
                        asms_main.push(asm);
                    }
                }

                asms_main.push("\tpop rbx\n".to_string());
                asms_main.push("\tadd rsp, 8\n".to_string());
                match bin.operator.token_type {
                    TokenType::Plus => asms_main.push(format!("\tadd {}, rbx\n", register)),
                    TokenType::Minus =>asms_main.push(format!("\tsub {}, rbx\n", register)),
                    TokenType::Star => asms_main.push(format!("\timul {}, rbx\n", register)),
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
                    asms_main.push(format!("\tmov {}, {}\n", register, val));
                }
                AnyMetadata::Number { value: NumberType::Float(val) } => {
                    asms_main.push(format!("\tmov {}, {:.9}\n", register, val));
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
                    self.data_section.push(format!("\tLC_{} db {}, 0\n", self.data_counter, (*value)).to_string());
                    self.data_section.push(format!("\tLC_len_{} equ {}\n", self.data_counter, (*value).len()).to_string());
                    asms_main.push(format!("\tlea rax, [rel LC_{}]\n", self.data_counter).to_string());
                }
                _ => unimplemented!("Only number literals supported for now"),
            },
            Expression::Call(c) => {
                let order = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                for i in 0..c.arguments.len() {
                    let value = self.compile_expression(&c.arguments[i], order[i])?;
                    for v in value {
                        asms_main.push(v);
                    }
                }
                asms_main.push(format!("\tcall {}\n", c.name));
            }
            _ => unimplemented!("Only number literals supported for now found: {:?}", expr),
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

