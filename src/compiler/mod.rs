use core::panic;
use std::{collections::HashMap, fs::File};
use crate::shared::{
    compiler_defaults::SIZES, errors::CompilerError, meta::{ AnyMetadata, NumberType }, parser_nodes::{Expression, ExpressionStatement, FunctionDeclaration, LiteralExpression, Program, ReturnStatement, Statement, TypedExpression, UnaryExpression, VarDeclarationStatement, VariableReassignmentStatement}, tokens::{Token, TokenType}
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
    pub text_section: Vec<String>,
    pub data_counter: usize,
    pub label_table: HashMap<String, Vec<String>>,
}

#[allow(dead_code)]
impl<'a> Compiler<'a> {
    pub fn new(ast: Program<'a>, out_file: &'a str) -> Result<Self, CompilerError> {
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
            } else if let Statement::ExternStatement(ex) = &statement {
                self.text_section.push(format!("\textern {}\n", ex.fx_name));
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
            Statement::VariableReassignmentStatement(vrs) => self.compile_variable_reassignment_statement(vrs),
            _ => {
                Err(CompilerError::UnexpectedStatement)
            }
        }
    }

    pub fn compile_variable_reassignment_statement(&mut self, stmt: &VariableReassignmentStatement<'a>) -> Result<Vec<String>, CompilerError> {
        let compiled_lhs = self.compile_address(&stmt.lhs, "rbx");
        let compiled_rhs = self.compile_expression(&stmt.rhs, "rdx");
        let mut asms_main = vec!["\n\t; VARIABLE REASSIGNMENT".to_string()];
        if let Ok(compiled_lhs) =  compiled_lhs {
            for lhs_asm in compiled_lhs {
                asms_main.push(lhs_asm);
            }

            if let Ok(compiled_rhs) =  compiled_rhs {
                for rhs_asm in compiled_rhs {
                    asms_main.push(rhs_asm);
                }
                asms_main.push("\tmov [rbx], rdx\n".to_string());
                Ok(asms_main)
            } else {
                panic!("{:?}", compiled_rhs);
            }
        } else {
            panic!("{:?}", compiled_lhs);
        }
    }
    

    pub fn compile_variable_declaration_statement(&mut self, stmt: &VarDeclarationStatement<'a>) -> Result<Vec<String>, CompilerError> {
        let mut asms_main = vec!["\n\t; VARIABLE DECLARATION\n".to_string()];
        asms_main.extend(self.compile_expression(&stmt.value, "rax")?);
        match stmt.variable_type {
            TypedExpression::Integer => {
                self.current_stack_offset -= SIZES.d_int as isize;
                self.symbol_table.insert(stmt.name, Symbol {
                    offset: self.current_stack_offset,
                    size: SIZES.d_int
                });
                asms_main.push(format!("\tmov DWORD [rbp-{}], eax\n", self.current_stack_offset.abs()));
            },
            TypedExpression::Float => {
                self.current_stack_offset -= SIZES.d_float as isize;
                self.symbol_table.insert(stmt.name, Symbol {
                    offset: self.current_stack_offset,
                    size: SIZES.d_int
                });
                asms_main.push(format!("\tmov QWORD [rbp-{}], rax\n", self.current_stack_offset.abs()));
            },
            TypedExpression::String => {
                self.current_stack_offset -= SIZES.d_ptr as isize;
                self.symbol_table.insert(stmt.name, Symbol {
                    offset: self.current_stack_offset,
                    size: SIZES.d_ptr
                });
                asms_main.push(format!("\tmov QWORD [rbp-{}], rax\n", self.current_stack_offset.abs()));
            },
            TypedExpression::Pointer(_) => {
                self.current_stack_offset -= SIZES.d_ptr as isize;
                self.symbol_table.insert(stmt.name, Symbol {
                    offset: self.current_stack_offset,
                    size: SIZES.d_ptr
                });
                asms_main.push(format!("\tmov QWORD [rbp-{}], rax\n", self.current_stack_offset.abs()));
            },
            _ => {
                return Err(CompilerError::UnknownDataType);
            }
        }
        asms_main.push("\n".to_string());
        Ok(asms_main)
    }
    pub fn align_bytes(&self, bytes: usize, alignment: usize) -> usize {
        let rem = bytes%alignment;
        if rem > 0 {
            bytes + alignment - rem
        } else {
            bytes
        }
    }

    pub fn compile_function_declaration_statement(&mut self, stmt: &FunctionDeclaration<'a>) -> Result<(String, Vec<String>), CompilerError> {
        let old_sp = self.current_stack_offset;
        self.current_stack_offset = 0;
        let mut total_arg_size = 0;
        for ar in stmt.arguments.clone() {
            let arg_size = match ar.arg_type {
                TypedExpression::Integer => SIZES.d_int,
                TypedExpression::String => SIZES.d_ptr,
                TypedExpression::Pointer(_) => SIZES.d_ptr,
                TypedExpression::Void => SIZES.d_bool,
                TypedExpression::Float => SIZES.d_float
            };
            
            total_arg_size += arg_size;
        }

        total_arg_size += stmt.variable_size;

        let mut body_stmts = vec![
            "\tpush rbp\n".to_string(),
            "\tmov rbp, rsp\n".to_string(),
            format!("\tsub rsp, {}\n", self.align_bytes(total_arg_size, 16))
        ];
        if stmt.arity > 0 {
            let order = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
            let order_32_bit = ["edi", "esi", "edx", "ecx", "e8", "e9"];
            (0..stmt.arity).for_each(|i| {
                let (size, operand_size, register): (usize, &'a str, &'a str) = match stmt.arguments[i].arg_type {
                    TypedExpression::Integer => (4, "DWORD", order_32_bit[i]),
                    TypedExpression::Float => (8, "QWORD", order[i]),
                    TypedExpression::String => (8, "QWORD", order[i]),
                    TypedExpression::Pointer(_) => (8, "QWORD", order[i]),
                    _ => unimplemented!("Can not determine size of other things.")
                };
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
            let compiled = self.compile_statement(body_statement);
            if let Ok(value) = compiled{
                for compiled_stmt in value {
                    body_stmts.push(compiled_stmt);
                }
            } else {
                println!("{:?}", body_statement);
                panic!("{:?}", compiled);
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
            main_asm_for_return.push("\n\t; Return Statement\n".to_string());
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
            x.insert(0, "; Expression Statement\n".to_string());
            Ok(x)
        } else {
            compiled
        }
    }

    fn get_register_info(&self, register: &str) -> Option<(usize, &str, &str, &str, &str)> {
        // Returns (size_bytes, 64bit, 32bit, 16bit, 8bit)
        match register {
            "rax" => Some((8, "rax", "eax", "ax", "al")),
            "rbx" => Some((8, "rbx", "ebx", "bx", "bl")),
            "rcx" => Some((8, "rcx", "ecx", "cx", "cl")),
            "rdx" => Some((8, "rdx", "edx", "dx", "dl")),
            "rdi" => Some((8, "rdi", "edi", "di", "dil")),
            "rsi" => Some((8, "rsi", "esi", "si", "sil")),
            "r8" => Some((8, "r8", "r8d", "r8w", "r8b")),
            "r9" => Some((8, "r9", "r9d", "r9w", "r9b")),
            // Add more registers as needed
            _ => None,
        }
    }


    fn emit_address_of_variable(&mut self, var_name: &str, target_register: &str) -> Result<Vec<String>, CompilerError> {
        let s = self.symbol_table.get(var_name).unwrap();
        Ok(vec![format!("\n\tlea {}, [rbp{}]\n", target_register, s.offset)])
    }

    fn compile_address(&mut self, expr: &Expression<'a>, register: &'a str) -> Result<Vec<String>, CompilerError> {
        match expr {
            Expression::Literal(LiteralExpression {
                value: Token {
                    meta_data: AnyMetadata::Identifier { value: name },
                    ..
                },
                ..
            }) => {
                self.emit_address_of_variable(name, register)
            },
            Expression::Unary(UnaryExpression{ operator, value }) => {
                match operator.token_type {
                    TokenType::Ampersand => {
                        let mut result = self.compile_address(value, register)?;
                        result.push(format!("\tmov {}, [{}]\n", register, register));
                        Ok(result)
                    }
                    t => {
                        let mut result = self.compile_address(value, register)?;
                        result.push(format!("\tmov {}, [{}]\n", register, register));
                        Ok(result)
                    }
                }
            }
            _ => unimplemented!()
        }
    }

    fn compile_deref(&mut self, expr: &Expression<'a>, register: &'a str) -> Result<Vec<String>, CompilerError> {
        match expr {
            Expression::Literal(LiteralExpression {
                value: Token {
                    meta_data: AnyMetadata::Identifier{..},
                    ..
                },
                ..
            }) => {
                let mut res = self.compile_expression(expr, register)?;
                res.push(format!("\tmov {}, [{}]\n", register, register));
                Ok(res)
            },
            Expression::Unary(UnaryExpression{ operator, value }) => {
                match operator.token_type {
                    TokenType::Ampersand => {
                        let mut result = self.compile_address(value, register)?;
                        result.push(format!("mov {}, [{}]", register, register));
                        Ok(result)
                    }
                    TokenType::Star => {
                        let mut res = self.compile_expression(expr, register)?;
                        res.push(format!("\tmov {}, [{}]\n", register, register));
                        Ok(res)
                    }
                    _ => unimplemented!()
                }
            }
            _ => unimplemented!()
        }
    }

    pub fn compile_expression(&mut self, expr: &Expression<'a>, register: &'a str) -> Result<Vec<String>, CompilerError> {
        println!("expr: {:?}, register: {:?}", expr, register);
        let mut asms_main = vec![];
        match expr {
            Expression::Binary(bin) => {
                if let Ok(compiled_asms) = self.compile_expression(&bin.left, register) {
                    for asm in compiled_asms {
                        asms_main.push(asm);
                    }
                }

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
                        asms_main.push("\tmov rax, rbx\n".to_string());
                        asms_main.push("\txor rdx, rdx\n".to_string());
                        asms_main.push(format!("\tdiv {}\n", register));
                        asms_main.push(format!("\tmov {}, rax\n", register));
                    }
                    _ => unimplemented!("Operator not implemented"),
                }
            }

            Expression::Literal(lit) => match &lit.value.meta_data {
                AnyMetadata::Number { value: NumberType::Integer(val) } => {
                    dbg!(format!("\tmov {}, {}\n", register, val));
                    asms_main.push(format!("\tmov {}, {}\n", register, val));
                }
                AnyMetadata::Number { value: NumberType::Float(val) } => {
                    asms_main.push(format!("\tmov {}, {:.9}\n", register, val));
                }
                AnyMetadata::Identifier { value } => {
                    let variable_symbol = self.symbol_table.get(value);
                    match variable_symbol {
                        Some(s) => {
                            // Get target register info
                            let reg_info = self.get_register_info(register)
                                .ok_or_else(|| {
                                    panic!("Unsupported register: {}", register);
                                })?;

                            let (target_reg_size, reg_64, reg_32, reg_16, reg_8) = reg_info;

                            // Check if we can fit the variable size into the target register
                            if s.size > target_reg_size {
                                panic!(
                                    "Size mismatch: Variable '{}' has size {} bytes, but target register '{}' can only hold {} bytes",
                                    value, s.size, register, target_reg_size
                                );
                            }

                            // Generate appropriate mov instruction based on variable size
                            match s.size {
                                4 => {
                                    asms_main.push(format!("\tmov {}, DWORD [rbp{}]\n", reg_32, s.offset));
                                    // If target is 64-bit register, the upper 32 bits are automatically cleared
                                }
                                8 => {
                                    if target_reg_size < 8 {
                                        panic!(
                                            "Size mismatch: Variable '{}' is 8 bytes but target register '{}' is only {} bytes",
                                            value, register, target_reg_size
                                        );
                                    }
                                    asms_main.push(format!("\tmov {}, QWORD [rbp{}]\n", reg_64, s.offset));
                                }
                                2 => {
                                    asms_main.push(format!("\tmov {}, WORD [rbp{}]\n", reg_16, s.offset));
                                    // Zero-extend to full register size if needed
                                    if target_reg_size > 2 {
                                        asms_main.push(format!("\tmovzx {}, {}\n", reg_64, reg_16));
                                    }
                                }
                                1 => {
                                    asms_main.push(format!("\tmov {}, BYTE [rbp{}]\n", reg_8, s.offset));
                                    // Zero-extend to full register size if needed
                                    if target_reg_size > 1 {
                                        asms_main.push(format!("\tmovzx {}, {}\n", reg_64, reg_8));
                                    }
                                }
                                _ => {
                                    panic!("Unsupported variable size: {} bytes for variable '{}'", s.size, value);
                                }
                            }
                        }
                        None => {
                            panic!("Unknown variable: '{}'", value);
                        }
                    }
                }
                AnyMetadata::String { value } => {
                    self.data_section.push(format!("\tLC_{} db {}, 0\n", self.data_counter, (*value)).to_string());
                    self.data_section.push(format!("\tLC_len_{} equ {}\n", self.data_counter, (*value).len()).to_string());
                    asms_main.push(format!("\tlea {}, [rel LC_{}]\n", register, self.data_counter).to_string());
                    self.data_counter += 1; // Don't forget to increment!
                }
                _ => unimplemented!("Only number literals supported for now"),
            }
            Expression::Call(c) => {
                let order = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                dbg!(&c);
                (0..c.arguments.len()).for_each(|i| {
                    println!("arg: {:?}, register: {:?}", &c.arguments[i], order[i]);
                    let value = self.compile_expression(&c.arguments[i], order[i]).expect("WTF");
                    for v in value {
                        asms_main.push(v);
                    }
                });
                asms_main.push("\txor rax, rax\n".to_string());
                asms_main.push(format!("\tcall {}\n", c.name));
                asms_main.push(format!("\tmov {}, rax\n", register));
            }
            Expression::Unary(u) => {
                match u.operator.token_type {
                    TokenType::Ampersand => {
                        return self.compile_address(&u.value, register)
                    }
                    TokenType::Star => {
                        return self.compile_deref(&u.value, register)
                    }
                    _ => unimplemented!()
                }
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

