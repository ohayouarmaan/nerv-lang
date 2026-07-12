use core::panic;
use std::{collections::HashMap, fs::File};
use crate::shared::{
    compiler_defaults::SIZES, errors::CompilerError, meta::{ AnyMetadata, NumberType }, parser_nodes::{Expression, ExpressionStatement, FieldAccessExpression, FunctionDeclaration, LiteralExpression, Program, ReturnStatement, Statement, StructDeclaration, StructLiteralExpression, TypedExpression, UnaryExpression, VarDeclarationStatement, VariableReassignmentStatement}, tokens::{Token, TokenType}
};

pub struct Symbol {
    pub offset: isize,
    pub size: usize,
    pub var_type: TypedExpression
}

pub enum SupportedTargets {
    Linux,
    Mac
}

#[cfg(target_os = "macos")]
pub fn getCurrentTarget() -> SupportedTargets {
    SupportedTargets::Mac
}

#[cfg(target_os = "linux")]
pub fn getCurrentTarget() -> SupportedTargets {
    SupportedTargets::Linux
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
    pub current_target: SupportedTargets,
    pub custom_types: HashMap<String, TypedExpression>,
    pub struct_defs: HashMap<String, StructDef>
}

#[derive(Debug, Clone)]
pub struct StructFieldDef {
    pub name: String,
    pub field_type: TypedExpression,
    pub offset: usize,
    pub size: usize,
    pub align: usize
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<StructFieldDef>,
    pub size: usize,
    pub align: usize
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
        let current_target = getCurrentTarget();

        Ok(Self {
            prog: ast,
            file_handler: file,
            asm,
            data_section,
            text_section,
            symbol_table: HashMap::new(),
            current_stack_offset: 0,
            data_counter: 0,
            label_table: HashMap::new(),
            current_target,
            custom_types: HashMap::new(),
            struct_defs: HashMap::new()
        })
    }

    pub fn compile(&mut self) -> Result<(), CompilerError> {
        let progs = self.prog.stmts.clone();
        for statement in progs {
            if let Statement::FunctionDeclaration(fx) = &statement {
                let compiled_fx = self.compile_function_declaration_statement(fx);
                if let Ok((function_name, body)) = compiled_fx {
                    let mut fx_name: String = "_".to_string();
                    if let SupportedTargets::Mac = self.current_target {
                        fx_name.push_str(&function_name);
                    } else {
                        fx_name = function_name;
                    }
                    self.text_section.push(format!("\tglobal {}\n", fx_name).to_string());
                    self.label_table.insert(fx_name, body);
                } else {
                    println!("Err: {:?}", compiled_fx);
                }
            } else if let Statement::ExternStatement(ex) = &statement {
                let mut fx_name: String = "_".to_string();
                if let SupportedTargets::Mac = self.current_target {
                    fx_name.push_str(ex.fx_name);
                } else {
                    fx_name = ex.fx_name.to_string()
                };
                self.text_section.push(format!("\textern {}\n", fx_name));
            } else if let Statement::TypeDeclarationStatement(tds) = &statement {
                if let AnyMetadata::Identifier { value } = tds.alias.meta_data {
                    self.custom_types.insert(value.to_string(), tds.alias_for.clone());
                }
            } else if let Statement::StructDeclaration(sd) = &statement {
                let def = self.build_struct_def(sd.name, sd.fields.clone());
                self.struct_defs.insert(sd.name.to_string(), def);
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

    pub fn compile_user_defined_type(&self, ut: &TypedExpression) -> TypedExpression {
        match ut {
            TypedExpression::UserDefinedTypeAlias { alias_for, .. } => {
                self.compile_user_defined_type(&*alias_for)
            }
            TypedExpression::Pointer(inner) => {
                TypedExpression::Pointer(Box::new(self.compile_user_defined_type(inner)))
            }
            TypedExpression::Function { args, return_type } => {
                let resolved_args = args.iter()
                    .map(|arg| self.compile_user_defined_type(arg))
                    .collect();
                let resolved_return = self.compile_user_defined_type(return_type);
                TypedExpression::Function {
                    args: resolved_args,
                    return_type: Box::new(resolved_return)
                }
            }
            _ => {
                ut.clone()
            }
        }
    }

    fn type_size_align(&self, t: &TypedExpression) -> (usize, usize) {
        match t {
            TypedExpression::Integer => (SIZES.d_int, SIZES.d_int),
            TypedExpression::Float => (SIZES.d_float, SIZES.d_float),
            TypedExpression::String => (SIZES.d_ptr, SIZES.d_ptr),
            TypedExpression::Void => (SIZES.d_bool, SIZES.d_bool),
            TypedExpression::Pointer(_) => (SIZES.d_ptr, SIZES.d_ptr),
            TypedExpression::Function { .. } => (SIZES.d_ptr, SIZES.d_ptr),
            TypedExpression::Struct { name } => {
                let def = self.struct_defs.get(name)
                    .unwrap_or_else(|| panic!("Unknown struct type {}", name));
                (def.size, def.align)
            }
            TypedExpression::UserDefinedTypeAlias { alias_for, .. } => {
                let resolved = self.compile_user_defined_type(alias_for);
                self.type_size_align(&resolved)
            }
        }
    }

    fn allocate_stack_slot(&mut self, size: usize, align: usize) -> isize {
        self.current_stack_offset -= size as isize;
        let abs = self.current_stack_offset.abs() as usize;
        let rem = abs % align;
        if rem != 0 {
            self.current_stack_offset -= (align - rem) as isize;
        }
        self.current_stack_offset
    }

    fn calculate_stack_size_for_function(&self, stmt: &FunctionDeclaration<'a>) -> usize {
        let mut offset: isize = 0;
        for arg in &stmt.arguments {
            let arg_type = self.compile_user_defined_type(&arg.arg_type);
            let (size, align) = self.type_size_align(&arg_type);
            offset -= size as isize;
            let abs = offset.abs() as usize;
            let rem = abs % align;
            if rem != 0 {
                offset -= (align - rem) as isize;
            }
        }
        for st in &stmt.body.values {
            if let Statement::VarDeclaration(var) = st {
                let var_type = self.compile_user_defined_type(&var.variable_type);
                let (size, align) = self.type_size_align(&var_type);
                offset -= size as isize;
                let abs = offset.abs() as usize;
                let rem = abs % align;
                if rem != 0 {
                    offset -= (align - rem) as isize;
                }
            }
        }
        self.align_bytes(offset.abs() as usize, 16)
    }

    pub fn compile_variable_declaration_statement(&mut self, stmt: &VarDeclarationStatement<'a>) -> Result<Vec<String>, CompilerError> {
        let mut asms_main = vec!["\n\t; VARIABLE DECLARATION\n".to_string()];
        let resolved_type = self.compile_user_defined_type(&stmt.variable_type);
        if let TypedExpression::Struct { name } = &resolved_type {
            let (struct_size, struct_align) = {
                let struct_def = self.struct_defs.get(name)
                    .unwrap_or_else(|| panic!("Unknown struct type {}", name));
                (struct_def.size, struct_def.align)
            };
            let offset = self.allocate_stack_slot(struct_size, struct_align);
            self.symbol_table.insert(stmt.name, Symbol {
                offset,
                size: struct_size,
                var_type: resolved_type.clone()
            });
            if let Expression::StructLiteral(lit) = &stmt.value {
                self.emit_struct_literal_init(lit, offset, &mut asms_main)?;
            } else {
                return Err(CompilerError::UnknownDataType);
            }
        } else {
            asms_main.extend(self.compile_expression(&stmt.value, "rax")?);
            let (size, align) = self.type_size_align(&resolved_type);
            let offset = self.allocate_stack_slot(size, align);
            self.symbol_table.insert(stmt.name, Symbol {
                offset,
                size,
                var_type: resolved_type
            });
            match size {
                8 => asms_main.push(format!("\tmov QWORD [rbp{}], rax\n", offset)),
                4 => asms_main.push(format!("\tmov DWORD [rbp{}], eax\n", offset)),
                1 => asms_main.push(format!("\tmov BYTE [rbp{}], al\n", offset)),
                _ => return Err(CompilerError::UnknownDataType),
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

    pub fn get_size_from_type(&self, t: &TypedExpression) -> usize {
        match t {
            TypedExpression::Integer => SIZES.d_int,
            TypedExpression::String => SIZES.d_ptr,
            TypedExpression::Pointer(_) => SIZES.d_ptr,
            TypedExpression::Void => SIZES.d_bool,
            TypedExpression::Float => SIZES.d_float,
            TypedExpression::UserDefinedTypeAlias { alias_for, .. } => self.get_size_from_type(&alias_for),
            TypedExpression::Function { .. } => SIZES.d_ptr,
            TypedExpression::Struct { name } => {
                let def = self.struct_defs.get(name)
                    .unwrap_or_else(|| panic!("Unknown struct type {}", name));
                def.size
            }
        }
    }

    pub fn compile_function_declaration_statement(&mut self, stmt: &FunctionDeclaration<'a>) -> Result<(String, Vec<String>), CompilerError> {
        let old_sp = self.current_stack_offset;
        self.current_stack_offset = 0;
        let total_arg_size = self.calculate_stack_size_for_function(stmt);

        let mut body_stmts = vec![
            "\tpush rbp\n".to_string(),
            "\tmov rbp, rsp\n".to_string(),
            format!("\tsub rsp, {}\n", self.align_bytes(total_arg_size, 16))
        ];
        if stmt.arity > 0 {
            let order = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
            let order_32_bit = ["edi", "esi", "edx", "ecx", "e8", "e9"];
            (0..stmt.arity).for_each(|i| {
                let arg_type = self.compile_user_defined_type(&stmt.arguments[i].arg_type);
                let (size, align) = self.type_size_align(&arg_type);
                let (operand_size, register): (&'a str, &'a str) = match size {
                    4 => ("DWORD", order_32_bit[i]),
                    8 => ("QWORD", order[i]),
                    1 => ("BYTE", order[i]),
                    _ => unimplemented!("Can not determine size of other things.")
                };
                let offset = self.allocate_stack_slot(size, align);
                body_stmts.push(format!("\tmov {} [rbp{}], {}\n", operand_size, offset, register));
                self.symbol_table.insert(stmt.arguments[i].name, Symbol {
                    offset,
                    size,
                    var_type: arg_type
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

    fn infer_expression_type(&self, expr: &Expression<'a>) -> TypedExpression {
        match expr {
            Expression::Literal(LiteralExpression {
                value: Token { meta_data: AnyMetadata::Identifier { value }, .. },
                ..
            }) => {
                if let Some(sym) = self.symbol_table.get(value) {
                    sym.var_type.clone()
                } else {
                    panic!("Unknown variable: {}", value);
                }
            }
            Expression::Unary(UnaryExpression { operator, value }) => {
                match operator.token_type {
                    TokenType::Ampersand => {
                        let inner = self.infer_expression_type(value);
                        TypedExpression::Pointer(Box::new(inner))
                    }
                    TokenType::Star => {
                        if let TypedExpression::Pointer(inner) = self.infer_expression_type(value) {
                            *inner
                        } else {
                            panic!("Trying to deref non-pointer");
                        }
                    }
                    _ => panic!("Unsupported unary type inference")
                }
            }
            Expression::FieldAccess(fa) => {
                let target_type = self.infer_expression_type(&fa.target);
                if let TypedExpression::Struct { name } = target_type {
                    let def = self.struct_defs.get(&name)
                        .unwrap_or_else(|| panic!("Unknown struct type {}", name));
                    let field = def.fields.iter().find(|f| f.name == fa.field)
                        .unwrap_or_else(|| panic!("Unknown field {} for struct {}", fa.field, name));
                    field.field_type.clone()
                } else {
                    panic!("Field access on non-struct");
                }
            }
            Expression::StructLiteral(sl) => {
                TypedExpression::Struct { name: sl.name.to_string() }
            }
            _ => panic!("Unsupported expression type inference")
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
            Expression::FieldAccess(FieldAccessExpression { target, field, .. }) => {
                let mut result = self.compile_address(target, register)?;
                let target_type = self.infer_expression_type(target);
                if let TypedExpression::Struct { name } = target_type {
                    let def = self.struct_defs.get(&name)
                        .unwrap_or_else(|| panic!("Unknown struct type {}", name));
                    let field_def = def.fields.iter()
                        .find(|f| f.name == *field)
                        .unwrap_or_else(|| panic!("Unknown field {} for struct {}", field, name));
                    if field_def.offset > 0 {
                        result.push(format!("\tadd {}, {}\n", register, field_def.offset));
                    }
                    Ok(result)
                } else {
                    panic!("Field access on non-struct");
                }
            }
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

                asms_main.push("\tpop rcx\n".to_string());
                match bin.operator.token_type {
                    TokenType::Plus => asms_main.push(format!("\tadd {}, rcx\n", register)),
                    TokenType::Minus =>asms_main.push(format!("\tsub {}, rcx\n", register)),
                    TokenType::Star => asms_main.push(format!("\timul {}, rcx\n", register)),
                    TokenType::Slash => {
                        asms_main.push("\tmov rax, rcx\n".to_string());
                        asms_main.push("\txor rdx, rdx\n".to_string());
                        asms_main.push(format!("\tdiv {}\n", register));
                        asms_main.push(format!("\tmov {}, rax\n", register));
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
                            let mut function_name: String = "_".to_string();
                            if let SupportedTargets::Linux = self.current_target {
                                function_name = value.to_string();
                            } else {
                                function_name.push_str(value);
                            }
                            asms_main.push(format!("\tlea {}, [rel {}]\n", register, function_name));
                        }
                    }
                }
                AnyMetadata::String { value } => {
                    if (**value).contains("\\n") {
                        let mut splitted_by_new_line: Vec<&str> = value.split("\\n").collect();
                        if let Some(last) = splitted_by_new_line.last() && last == &"" {
                            splitted_by_new_line.pop();
                        }
                        let mut lc = String::from(format!("\tLC_{} db ", self.data_counter));
                        let mut x = String::new();
                        for d in splitted_by_new_line {
                            if(d != "") {
                                let newd = d.replace("\"", "");
                                x.push_str("\"");
                                x.push_str(&newd);
                                x.push_str("\"");
                            }
                            x.push_str(", 10");
                        }
                        x.push_str(", 0\n");
                        lc.push_str(&x.to_string());
                        self.data_section.push(lc);
                        self.data_section.push(format!("\tLC_len_{} equ {}\n", self.data_counter, (*value).len()).to_string());
                        asms_main.push(format!("\tlea {}, [rel LC_{}]\n", register, self.data_counter).to_string());
                        self.data_counter += 1; // Don't forget to increment!
                    } else {
                        self.data_section.push(format!("\tLC_{} db \"{}\", 0\n", self.data_counter, (*value)).to_string());
                        self.data_section.push(format!("\tLC_len_{} equ {}\n", self.data_counter, (*value).len()).to_string());
                        asms_main.push(format!("\tlea {}, [rel LC_{}]\n", register, self.data_counter).to_string());
                        self.data_counter += 1; // Don't forget to increment!
                    }
                }
                _ => unimplemented!("Only number literals supported for now"),
            }
            Expression::Call(c) => {
                let order = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                (0..c.arguments.len()).for_each(|i| {
                    let value = self.compile_expression(&c.arguments[i], order[i]).expect("WTF");
                    for v in value {
                        asms_main.push(v);
                    }
                });
                asms_main.push("\txor rax, rax\n".to_string());
                if let Expression::Literal(LiteralExpression { value: Token { meta_data: AnyMetadata::Identifier { value }, .. }, .. }) = &*c.callee {
                    if self.symbol_table.contains_key(value) {
                        asms_main.extend(self.compile_expression(&c.callee, "rax")?);
                        asms_main.push("\tcall rax\n".to_string());
                    } else {
                        let mut function_name: String = "_".to_string();
                        if let SupportedTargets::Linux = self.current_target {
                            function_name = value.to_string();
                        } else {
                            function_name.push_str(value);
                        }
                        asms_main.push(format!("\tcall {}\n", function_name));
                    }
                } else {
                    asms_main.extend(self.compile_expression(&c.callee, "rax")?);
                    asms_main.push("\tcall rax\n".to_string());
                }
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
            Expression::FieldAccess(_) => {
                let field_type = self.infer_expression_type(expr);
                let (size, _) = self.type_size_align(&field_type);
                let mut res = self.compile_address(expr, "rbx")?;
                let reg_info = self.get_register_info(register)
                    .ok_or_else(|| CompilerError::UnknownDataType)?;
                let (target_reg_size, reg_64, reg_32, _reg_16, reg_8) = reg_info;
                if size > target_reg_size {
                    return Err(CompilerError::UnknownDataType);
                }
                match size {
                    8 => res.push(format!("\tmov {}, QWORD [rbx]\n", reg_64)),
                    4 => res.push(format!("\tmov {}, DWORD [rbx]\n", reg_32)),
                    1 => {
                        res.push(format!("\tmov {}, BYTE [rbx]\n", reg_8));
                        if target_reg_size > 1 {
                            res.push(format!("\tmovzx {}, {}\n", reg_64, reg_8));
                        }
                    }
                    _ => return Err(CompilerError::UnknownDataType),
                }
                return Ok(res);
            }
            Expression::StructLiteral(_) => {
                return Err(CompilerError::UnknownDataType);
            }
            _ => unimplemented!("Only number literals supported for now found: {:?}", expr),
        }
        Ok(asms_main)
    }

    fn build_struct_def(&self, name: &str, fields: Vec<crate::shared::parser_nodes::StructField<'a>>) -> StructDef {
        let mut layout_fields = vec![];
        let mut offset = 0;
        let mut max_align = 1;
        for field in fields {
            let field_type = self.compile_user_defined_type(&field.field_type);
            let (size, align) = self.type_size_align(&field_type);
            if align > max_align {
                max_align = align;
            }
            let rem = offset % align;
            if rem != 0 {
                offset += align - rem;
            }
            layout_fields.push(StructFieldDef {
                name: field.name.to_string(),
                field_type,
                offset,
                size,
                align
            });
            offset += size;
        }
        let final_rem = offset % max_align;
        if final_rem != 0 {
            offset += max_align - final_rem;
        }
        StructDef {
            name: name.to_string(),
            fields: layout_fields,
            size: offset,
            align: max_align
        }
    }

    fn emit_struct_literal_init(&mut self, lit: &StructLiteralExpression<'a>, base_offset: isize, asms_main: &mut Vec<String>) -> Result<(), CompilerError> {
        let struct_fields = {
            let struct_def = self.struct_defs.get(lit.name)
                .unwrap_or_else(|| panic!("Unknown struct type {}", lit.name));
            struct_def.fields.clone()
        };
        for field in &lit.fields {
            let field_def = struct_fields.iter()
                .find(|f| f.name == field.name)
                .unwrap_or_else(|| panic!("Unknown field {} for struct {}", field.name, lit.name));
            let compiled_value = self.compile_expression(&field.value, "rax")?;
            for asm in compiled_value {
                asms_main.push(asm);
            }
            let field_offset = base_offset + field_def.offset as isize;
            match field_def.size {
                8 => asms_main.push(format!("\tmov QWORD [rbp{}], rax\n", field_offset)),
                4 => asms_main.push(format!("\tmov DWORD [rbp{}], eax\n", field_offset)),
                1 => asms_main.push(format!("\tmov BYTE [rbp{}], al\n", field_offset)),
                _ => return Err(CompilerError::UnknownDataType),
            }
        }
        Ok(())
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

