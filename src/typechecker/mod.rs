use std::collections::HashMap;

use crate::shared::{
    meta::AnyMetadata, parser_nodes::{
        BlockStatement, Expression, ExpressionStatement, ExternFunctionStatement, FieldAccessExpression, FunctionDeclaration, Program, ReturnStatement, Statement, StructDeclaration, StructLiteralExpression, TypeDeclarationStatement, TypedExpression, VarDeclarationStatement, VariableReassignmentStatement
    }, tokens::TokenType
};

pub struct TypeChecker<'a> {
    program: Program<'a>,
    env: TypeEnv,
}

#[derive(Debug, Clone)]
pub enum Types {
    Integer,
    Float,
    String,
    Void,
}

#[derive(Debug, Clone)]
pub enum Declaration<'a> {
    FunctionDeclaration(FunctionDeclaration<'a>),
}

#[derive(Debug, Clone)]
pub struct TypeEnv {
    return_type: Option<TypedExpression>,
    vars: HashMap<String, TypedExpression>,
    functions: HashMap<String, (TypedExpression, Vec<TypedExpression>)>,
    custom_types: HashMap<String, (TypedExpression)>,
    struct_defs: HashMap<String, StructDef>
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

impl<'a> TypeChecker<'a> {
    pub fn new(program: Program<'a>) -> Self {
        Self {
            program,
            env: TypeEnv {
                return_type: None,
                vars: HashMap::new(),
                functions: HashMap::new(),
                custom_types: HashMap::new(),
                struct_defs: HashMap::new()
            },
        }
    }

    pub fn check(&mut self) {
        self.start_type_checking(self.program.stmts.clone());
    }

    pub fn start_type_checking(&mut self, stmts: Vec<Statement<'a>>) {
        for x in stmts.clone() {
            if let Statement::FunctionDeclaration(t) = x {
                let mut args = vec![];
                for arg in t.arguments {
                    args.push(arg.arg_type);
                }
                self.env.functions.insert(t.name.to_string(), (t.return_type, args));
            }
        }
        for stmt in stmts {
            match stmt {
                Statement::VarDeclaration(var_decl) => self.type_check_var_declaration(var_decl),
                Statement::ExpressionStatement(expr_stmt) => self.type_check_expression_statement(expr_stmt),
                Statement::FunctionDeclaration(func_decl) => self.type_check_function_declaration(func_decl),
                Statement::BlockStatement(block_stmt) => self.type_check_block_statement(block_stmt),
                Statement::ReturnStatement(ret_stmt) => self.type_check_return_statement(ret_stmt),
                Statement::ExternStatement(ex) => self.type_check_extern_statement(ex),
                Statement::VariableReassignmentStatement(vrs) => self.type_check_reassignment_statement(vrs),
                Statement::TypeDeclarationStatement(tds) => self.check_type_declaration(&tds),
                Statement::StructDeclaration(sd) => self.check_struct_declaration(sd),
            }
        }
    }

    pub fn check_type_declaration(&mut self, tds: &TypeDeclarationStatement) {
        if let AnyMetadata::Identifier { value } = tds.alias.meta_data {
            self.env.custom_types.insert(value.to_string(), tds.alias_for.clone());
        }
    }

    pub fn check_struct_declaration(&mut self, sd: StructDeclaration<'a>) {
        let def = self.build_struct_def(sd.name, sd.fields);
        self.env.struct_defs.insert(sd.name.to_string(), def);
    }

    pub fn type_check_reassignment_statement(&self, vrs: VariableReassignmentStatement<'a>) {
        let ldata_type = self.eval_expression(&vrs.lhs);
        let rdata_type = self.eval_expression(&vrs.rhs);
        if ldata_type != rdata_type {
            panic!("Left Hand Side is of type {:?} and you're trying to assign {:?}", ldata_type, rdata_type);
        }
    }

    pub fn compile_user_defined_type(&self, user_defined_type: TypedExpression) -> TypedExpression {
        match user_defined_type {
            TypedExpression::UserDefinedTypeAlias { identifier, .. } => {
                if let Some(ut) = self.env.custom_types.get(&identifier) {
                    self.compile_user_defined_type(ut.clone())
                } else {
                    panic!("Unknown Type");
                }
            },
            TypedExpression::Pointer(x) => {
                TypedExpression::Pointer(Box::new(self.compile_user_defined_type(*x)))
            }
            TypedExpression::Function { args, return_type } => {
                let resolved_args = args.into_iter()
                    .map(|arg| self.compile_user_defined_type(arg))
                    .collect();
                let resolved_return = self.compile_user_defined_type(*return_type);
                TypedExpression::Function {
                    args: resolved_args,
                    return_type: Box::new(resolved_return)
                }
            }
            _ => user_defined_type
        }
    }

    pub fn type_check_extern_statement(&mut self, ex: ExternFunctionStatement<'a>)  {

        let mut args = vec![];
        for param in &ex.fx_sig.args {
            args.push(self.compile_user_defined_type(param.clone()));
        }

        let return_type = ex.fx_sig.return_type;
        self.env.return_type = Some(return_type.clone());
        self.env.functions.insert(ex.fx_sig.fx_name.to_string(), (return_type, args));
    }

    pub fn type_check_var_declaration(&mut self, v: VarDeclarationStatement<'a>) {
        // Assuming v has a name and a declared type
        let var_name = v.name;
        let var_type = self.compile_user_defined_type(v.variable_type);

        let expr_type = self.compile_user_defined_type(self.eval_expression(&v.value));
        if expr_type != var_type {
            panic!("Type mismatch in variable declaration: expected {:?}, got {:?}", var_type, expr_type);
        }

        self.env.vars.insert(var_name.to_string(), var_type);
    }

    pub fn type_check_return_statement(&self, r: ReturnStatement<'a>) {
        let expected_return_type = self.compile_user_defined_type(self.env.return_type.clone().unwrap());
        let expr_type = self.compile_user_defined_type(self.eval_expression(&r.value));

        if expr_type != expected_return_type {
            panic!(
                "Return type mismatch: expected {:?}, got {:?} {}:{}",
                expected_return_type, expr_type, r.position.line, r.position.column
            );
        }
    }

    pub fn type_check_block_statement(&mut self, b: BlockStatement<'a>) {
        let previous_env = self.env.vars.clone();

        for stmt in b.values {
            self.start_type_checking(vec![stmt]);
        }

        self.env.vars = previous_env;
    }

    pub fn type_check_expression_statement(&self, e: ExpressionStatement<'a>) {
        self.eval_expression(&e.value);
    }

    pub fn type_check_function_declaration(&mut self, f: FunctionDeclaration<'a>) {
        self.type_check_function(f);
    }

    pub fn type_check_function(&mut self, fx: FunctionDeclaration<'a>) {
        let return_type = self.compile_user_defined_type(fx.return_type);
        let old_env = self.env.clone();
        self.env.return_type = Some(return_type.clone());

        let mut args = vec![];
        for param in &fx.arguments {
            args.push(param.arg_type.clone());
            self.env
                .vars
                .insert(param.name.to_string(), self.compile_user_defined_type(param.arg_type.clone()));
        }
        
        self.type_check_block_statement(fx.body);
        self.env = old_env;
        self.env.functions.insert(fx.name.to_string(), (return_type, args));
    }

    fn eval_custom_type(&self, token_identifier: &'a str) -> &TypedExpression {
        if let Some(first_alias_for)  = self.env.custom_types.get(token_identifier) {
            if let TypedExpression::UserDefinedTypeAlias { identifier, ..  } = first_alias_for {
                return self.eval_custom_type(identifier);
            }
            return first_alias_for;
        }
        panic!("Invalid User Defined Type: {:?}", token_identifier);
    }

    fn eval_expression(&self, expr: &Expression<'a>) -> TypedExpression {
        match expr {
            Expression::Binary(binary_expression) => {
                let lhs = self.eval_expression(&binary_expression.left);
                let rhs = self.eval_expression(&binary_expression.right);
                match (binary_expression.operator.token_type, lhs, rhs) {
                    (TokenType::Plus | TokenType::Minus | TokenType::Star, TypedExpression::Integer, TypedExpression::Integer) => {
                        TypedExpression::Integer
                    },
                    (TokenType::Plus | TokenType::Minus | TokenType::Star, TypedExpression::Integer, TypedExpression::Float) => {
                        TypedExpression::Float
                    },
                    (TokenType::Plus | TokenType::Minus | TokenType::Star, TypedExpression::Float, TypedExpression::Integer) => {
                        TypedExpression::Float
                    },
                    (TokenType::Plus | TokenType::Minus | TokenType::Star, TypedExpression::Float, TypedExpression::Float) => {
                        TypedExpression::Float
                    },
                    (TokenType::EqualEqual, _, _) => {
                        TypedExpression::Float
                    },
                    (TokenType::Slash, _, _) => {
                        TypedExpression::Float
                    },
                    _ => panic!("Type error in binary expression")
                }
            },
            Expression::Unary(u) => {
                match u.operator.token_type {
                    TokenType::Ampersand => {
                        let x = self.eval_expression(&u.value);
                        TypedExpression::Pointer(Box::new(x))
                    }
                    TokenType::Star => {
                        if let TypedExpression::Pointer(x) = self.eval_expression(&u.value) {
                            *x
                        } else {
                            panic!("You're trying to deref a {:?} type", self.eval_expression(&u.value));
                        }
                    }
                    _ => unimplemented!()
                }
            },
            Expression::Call(c) => {
                let callee_type = self.compile_user_defined_type(self.eval_expression(&c.callee));
                if let TypedExpression::Function { args, return_type } = callee_type {
                    if args.len() != c.arguments.len() {
                        panic!("Expected {} arguments got {}", args.len(), c.arguments.len());
                    }
                    (0..c.arguments.len()).for_each(|i| {
                        let arg = c.arguments[i].clone();
                        let arg_type = self.compile_user_defined_type(self.eval_expression(&arg));
                        if arg_type != args[i] {
                            panic!("Expected argument type to be {:?} instead got {:?} {}:{}", args[i], arg_type, c.position.line, c.position.column);
                        }
                    });
                    *return_type
                } else {
                    panic!("Trying to call a non-function type.");
                }
            }
            Expression::Literal(literal_expression) => {
                match literal_expression.value.token_type {
                    TokenType::Integer => TypedExpression::Integer,
                    TokenType::String => TypedExpression::String,
                    TokenType::Float => TypedExpression::Float,
                    TokenType::Void => TypedExpression::Void,
                    TokenType::Identifier => {
                        if let AnyMetadata::Identifier { value } = literal_expression.value.meta_data {
                            if let Some(variable_type) = self.env.vars.get(value) {
                                if let TypedExpression::UserDefinedTypeAlias { identifier, .. } = variable_type {
                                    return self.eval_custom_type(identifier).clone();
                                }
                                variable_type.clone()
                            } else if let Some((return_type, args)) = self.env.functions.get(value) {
                                TypedExpression::Function {
                                    args: args.clone(),
                                    return_type: Box::new(return_type.clone())
                                }
                            } else {
                                panic!("Unknown Variable {}:{}", literal_expression.value.position.line, literal_expression.value.position.column);
                            }
                        } else {
                            panic!("Unknown Variable {}:{}", literal_expression.value.position.line, literal_expression.value.position.column);
                        }
                    }
                    _ => {
                        panic!("Unknown Literal Expression: {:?} {}:{}", literal_expression.value, literal_expression.value.position.line, literal_expression.value.position.column);
                    }
                }
            },
            Expression::StructLiteral(sl) => {
                let struct_def = self.env.struct_defs.get(sl.name)
                    .unwrap_or_else(|| panic!("Unknown struct type {}", sl.name));
                if sl.fields.len() != struct_def.fields.len() {
                    panic!("Struct literal missing fields {}", sl.name);
                }
                for field in &sl.fields {
                    let expected_field = struct_def.fields.iter().find(|f| f.name == field.name)
                        .unwrap_or_else(|| panic!("Unknown field {} for struct {}", field.name, sl.name));
                    let value_type = self.compile_user_defined_type(self.eval_expression(&field.value));
                    if value_type != expected_field.field_type {
                        panic!("Struct field {} expects {:?} got {:?}", field.name, expected_field.field_type, value_type);
                    }
                }
                TypedExpression::Struct { name: sl.name.to_string() }
            }
            Expression::FieldAccess(fa) => {
                let target_type = self.compile_user_defined_type(self.eval_expression(&fa.target));
                if let TypedExpression::Struct { name } = target_type {
                    let struct_def = self.env.struct_defs.get(&name)
                        .unwrap_or_else(|| panic!("Unknown struct type {}", name));
                    if let Some(field) = struct_def.fields.iter().find(|f| f.name == fa.field) {
                        field.field_type.clone()
                    } else {
                        panic!("Unknown field {} for struct {}", fa.field, name);
                    }
                } else {
                    panic!("Field access on non-struct type");
                }
            }
        }
    }

    fn type_size_align(&self, t: &TypedExpression) -> (usize, usize) {
        match t {
            TypedExpression::Integer => (4, 4),
            TypedExpression::Float => (8, 8),
            TypedExpression::String => (8, 8),
            TypedExpression::Void => (1, 1),
            TypedExpression::Pointer(_) => (8, 8),
            TypedExpression::Function { .. } => (8, 8),
            TypedExpression::Struct { name } => {
                let def = self.env.struct_defs.get(name)
                    .unwrap_or_else(|| panic!("Unknown struct type {}", name));
                (def.size, def.align)
            }
            TypedExpression::UserDefinedTypeAlias { identifier, .. } => {
                let resolved = self.eval_custom_type(identifier).clone();
                self.type_size_align(&resolved)
            }
        }
    }

    fn build_struct_def(&self, name: &str, fields: Vec<crate::shared::parser_nodes::StructField<'a>>) -> StructDef {
        let mut layout_fields = vec![];
        let mut offset = 0;
        let mut max_align = 1;
        for field in fields {
            let field_type = self.compile_user_defined_type(field.field_type);
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
}
