use std::{collections::HashMap, hash::Hash};

use crate::shared::{
    meta::AnyMetadata, parser_nodes::{
        BlockStatement, Expression, ExpressionStatement, ExternFunctionStatement, FunctionDeclaration, ParserTypedStructField, Program, ReturnStatement, Statement, StructDefinition, TypeDeclarationStatement, TypedExpression, VarDeclarationStatement, VariableReassignmentStatement
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
pub struct TypedStructField {
    pub field: String,
    pub item_type: TypedExpression
}

#[derive(Debug, Clone)]
pub struct TypeEnv {
    return_type: Option<TypedExpression>,
    vars: HashMap<String, TypedExpression>,
    functions: HashMap<String, (TypedExpression, Vec<TypedExpression>)>,
    structs: HashMap<String, TypedStructField>,
    custom_types: HashMap<String, (TypedExpression)>
}

impl<'a> TypeChecker<'a> {
    pub fn new(program: Program<'a>) -> Self {
        Self {
            program,
            env: TypeEnv {
                return_type: None,
                vars: HashMap::new(),
                functions: HashMap::new(),
                structs: HashMap::new(),
                custom_types: HashMap::new(),
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
                Statement::StructDeclarationStatement(sds) => self.type_check_struct_definition(sds),
            }
        }
    }

    pub fn type_check_struct_definition(&mut self, sds: StructDefinition) {
        for field in sds.fields {
            self.env.structs.insert(sds.name.to_string(), TypedStructField { field: field.name.to_string(), item_type: field.item_type });
        };
    }

    pub fn check_type_declaration(&mut self, tds: &TypeDeclarationStatement) {
        if let AnyMetadata::Identifier { value } = tds.alias.meta_data {
            self.env.custom_types.insert(value.to_string(), tds.alias_for.clone());
        }
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
                        if let Some((result, args)) = self.env.functions.get(c.name) {
                            (0..c.arguments.len()).for_each(|i| {
                                let arg = c.arguments[i].clone();
                                let arg_type = self.compile_user_defined_type(self.eval_expression(&arg));
                                if arg_type != args[i] {
                                    panic!("Expected argument type to be {:?} instead got {:?} {}:{}", args[i], arg_type, c.position.line, c.position.column);
                                }
                            });
                            result.clone()
                        } else {
                            panic!("No such function exists.");
                        }
                    },
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
            Expression::Struct(struct_expression) => {
                TypedExpression::Struct {
                    fields: struct_expression.fields.iter().map(|s| ParserTypedStructField {
                        field_type: Box::new(self.eval_expression(&s.field_value)),
                        field_name: s.field_name.to_string(),
                    }).collect()
                }
            },
        }
    }
}
