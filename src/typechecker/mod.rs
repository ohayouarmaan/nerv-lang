use std::collections::HashMap;

use crate::shared::{
    meta::AnyMetadata, parser_nodes::{
        BlockStatement, Expression, ExpressionStatement, FunctionDeclaration, Program, ReturnStatement, Statement, VarDeclarationStatement
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
    return_type: Option<TokenType>,
    vars: HashMap<String, TokenType>,
    functions: HashMap<String, (TokenType, Vec<TokenType>)>
}

impl<'a> TypeChecker<'a> {
    pub fn new(program: Program<'a>) -> Self {
        Self {
            program,
            env: TypeEnv {
                return_type: None,
                vars: HashMap::new(),
                functions: HashMap::new(),
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
            }
        }
    }

    pub fn type_check_var_declaration(&mut self, v: VarDeclarationStatement<'a>) {
        // Assuming v has a name and a declared type
        let var_name = v.name;
        let var_type = v.variable_type;

        let expr_type = self.eval_expression(&v.value);
        if expr_type != var_type {
            panic!("Type mismatch in variable declaration: expected {:?}, got {:?}", var_type, expr_type);
        }

        self.env.vars.insert(var_name.to_string(), var_type);
    }

    pub fn type_check_return_statement(&self, r: ReturnStatement<'a>) {
        let expected_return_type = self.env.return_type;
        let expr_type = self.eval_expression(&r.value);

        if Some(expr_type) != expected_return_type {
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
        let return_type = fx.return_type;
        let old_env = self.env.clone();
        self.env.return_type = Some(return_type);

        let mut args = vec![];
        for param in &fx.arguments {
            args.push(param.arg_type);
            self.env
                .vars
                .insert(param.name.to_string(), param.arg_type);
        }
        
        self.type_check_block_statement(fx.body);
        self.env = old_env;
        self.env.functions.insert(fx.name.to_string(), (return_type, args));
    }

    fn eval_expression(&self, expr: &Expression<'a>) -> TokenType {
        match expr {
            Expression::Binary(binary_expression) => {
                let lhs = self.eval_expression(&binary_expression.left);
                let rhs = self.eval_expression(&binary_expression.right);
                match (binary_expression.operator.token_type, lhs, rhs) {
                    (TokenType::Plus | TokenType::Minus | TokenType::Star, TokenType::DInteger, TokenType::DInteger) => {
                        TokenType::DInteger
                    },
                    (TokenType::Plus | TokenType::Minus | TokenType::Star, TokenType::DInteger, TokenType::DFloat) => {
                        TokenType::DFloat
                    },
                    (TokenType::Plus | TokenType::Minus | TokenType::Star, TokenType::DFloat, TokenType::DInteger) => {
                        TokenType::DFloat
                    },
                    (TokenType::Plus | TokenType::Minus | TokenType::Star, TokenType::DFloat, TokenType::DFloat) => {
                        TokenType::DFloat
                    },
                    (TokenType::EqualEqual, _, _) => {
                        TokenType::DInteger
                    },
                    _ => panic!("Type error in binary expression")
                }
            },
            Expression::Unary(_) => TokenType::DInteger,
            Expression::Call(c) => {
                if let Some((result, args)) = self.env.functions.get(c.name) {
                    (0..c.arguments.len()).for_each(|i| {
                        let arg = c.arguments[i].clone();
                        let arg_type = self.eval_expression(&arg);
                        if arg_type != args[i] {
                            panic!("Expected argument type to be {:?} instead got {:?} {}:{}", args[i], arg_type, c.position.line, c.position.column);
                        }
                    });
                    *result
                } else {
                    panic!("No such function exists.");
                }
            },
            Expression::Literal(literal_expression) => {
                match literal_expression.value.token_type {
                    TokenType::Integer => TokenType::DInteger,
                    TokenType::String => TokenType::DString,
                    TokenType::Float => TokenType::DFloat,
                    TokenType::Void => TokenType::DVoid,
                    TokenType::Identifier => {
                        if let AnyMetadata::Identifier { value } = literal_expression.value.meta_data {
                            if let Some(variable_type) = self.env.vars.get(value) {
                                *variable_type
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
        }
    }
}
