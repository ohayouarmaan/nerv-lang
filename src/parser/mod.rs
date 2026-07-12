use crate::{
    lexer::Lexer,
    shared::{
        meta::AnyMetadata, parser_nodes::{
            Argument, BinaryExpression, BlockStatement, CallExpression, Expression, ExpressionStatement, ExternFunctionStatement, FieldAccessExpression, FunctionDeclaration, FunctionSignatureDeclaration, LiteralExpression, Program, ReturnStatement, Statement, StructDeclaration, StructField, StructLiteralExpression, StructLiteralField, TypeDeclarationStatement, TypedExpression, UnaryExpression, VarDeclarationStatement, VariableReassignmentStatement
        }, positions::Position, tokens::{
            Token,
            TokenType
        }
    }
};
use core::panic;
use std::{collections::HashMap, iter::Peekable};

#[allow(dead_code)]
pub struct Parser<'a> {
    pub lexer: Peekable<Lexer<'a>>,
    pub previous_token: Option<Token<AnyMetadata<'a>>>,
    pub custom_types: HashMap<String, TypedExpression>
}


#[allow(dead_code)]
impl<'a> Parser<'a> {
    pub fn new(source_code: &'a str) -> Self {
        let lexer = Lexer::new(source_code).peekable();
        Self {
            lexer,
            previous_token: None,
            custom_types: HashMap::new()
        }
    }

    pub fn parse(&mut self) -> Program<'a> {
        let mut stmts: Vec<Statement> = vec![];
        while self.lexer.peek().is_some() {
            stmts.push(self.parse_statement());
        }

        Program {
            stmts
        }
    }

    fn tt_to_typed(&mut self, t: Token<AnyMetadata>) -> TypedExpression {
        match t.token_type {
            TokenType::DVoid => {
                TypedExpression::Void
            },
            TokenType::DInteger => {
                TypedExpression::Integer
            },
            TokenType::DString => {
                TypedExpression::String
            },
            TokenType::DFloat => {
                TypedExpression::Float
            },
            TokenType::Identifier => {
                if let AnyMetadata::Identifier { value } = t.meta_data {
                    return self.custom_types.get(value).expect("Unknown Type, you might want to define it before hand.").clone();
                }
                panic!("UNREACHABLE")
            },
            TokenType::Ampersand => {
                let pointer_to = self.parse_type_expression();
                TypedExpression::Pointer(Box::new(pointer_to))
            },
            TokenType::Fun => {
                self.consume(TokenType::LeftParen);
                let mut args = vec![];
                if !self.match_tokens(&[TokenType::RightParen]) {
                    loop {
                        args.push(self.parse_type_expression());
                        if self.match_tokens(&[TokenType::Comma]) {
                            continue;
                        }
                        self.consume(TokenType::RightParen);
                        break;
                    }
                }
                self.consume(TokenType::Arrow);
                let return_type = self.parse_type_expression();
                TypedExpression::Function {
                    args,
                    return_type: Box::new(return_type)
                }
            }
            _ => {
                panic!("");
            }
        }
    }

    pub fn compile_user_defined_type(&self, user_defined_type: TypedExpression) -> TypedExpression {
        match user_defined_type {
            TypedExpression::UserDefinedTypeAlias { identifier, .. } => {
                if let Some(ut) = self.custom_types.get(&identifier) {
                    ut.clone()
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

    fn parse_statement(&mut self) -> Statement<'a> {
        if let Some(current_token) = self.lexer.peek() {
            let starting_position = current_token.position;
            match current_token.token_type {
                TokenType::Dec => {

                    // Variable Declaration Statement
                    self.consume(TokenType::Dec);
                    if let Some(t) = self.lexer.next() {
                        if t.token_type != TokenType::Identifier {
                            panic!("Expected a Identifier after 'dec' {:?}", self.previous_token);
                        }
                        if let AnyMetadata::Identifier{ value } = t.meta_data {
                            let data_type = self.parse_type_expression();
                            let after_data_type = self.lexer.next();
                            if let Some(Token { token_type: TokenType::Equal, .. }) = after_data_type {
                                let expr = self.parse_expression();
                                self.consume(TokenType::Semicolon);
                                return Statement::VarDeclaration(VarDeclarationStatement {
                                    value: expr,
                                    name: value,
                                    variable_type: data_type,
                                    position: starting_position
                                });
                            } else {
                                println!("UNEXPECTED: {:?}", after_data_type);
                            }
                        }
                    }
                }
                TokenType::At => {
                    self.consume(TokenType::At);
                    return self.parse_function(starting_position);
                }
                TokenType::Return => {
                    self.consume(TokenType::Return);
                    let value = self.parse_expression();
                    self.consume(TokenType::Semicolon);
                    return Statement::ReturnStatement(ReturnStatement {
                        value,
                        position: starting_position
                    });
                }
                TokenType::Extern => {
                    self.consume(TokenType::Extern);
                    let fx_name = if self.match_tokens(&[TokenType::Identifier]) {
                        let fx_token = self.previous_token.clone().unwrap();
                        if let AnyMetadata::Identifier { value } = fx_token.meta_data {
                            value
                        } else {
                            panic!("Expected {:?} got {:?}", TokenType::Identifier, fx_token.token_type);
                        }
                    } else {
                        panic!("Expected Identifier got {:?}", self.previous_token);
                    };

                    self.consume(TokenType::LeftParen);
                    let mut args: Vec<TypedExpression> = vec![];
                    while !self.match_tokens(&[TokenType::RightParen]) {
                        args.push(self.parse_type_expression());
                        if self.match_tokens(&[TokenType::Comma]) {
                            continue;
                        }
                    }

                    let return_type: TypedExpression = self.parse_type_expression();

                    self.consume(TokenType::Semicolon);
                    let fx_sig = FunctionSignatureDeclaration {
                        fx_name,
                        args,
                        return_type,
                    };
                    return Statement::ExternStatement(ExternFunctionStatement {
                        fx_name,
                        fx_sig
                    });
                }

                TokenType::Type => {
                    self.consume(TokenType::Type);
                    self.consume(TokenType::Identifier);
                    let alias = self.previous_token.expect("UNREACHABLE");
                    self.consume(TokenType::Colon);
                    if self.match_tokens(&[TokenType::DInteger, TokenType::DString, TokenType::DFloat, TokenType::DVoid, TokenType::Identifier]){
                        let alias_for = self.tt_to_typed(self.previous_token.expect("UNREACHABLE"));
                        self.consume(TokenType::Semicolon);
                        let t = TypeDeclarationStatement {
                            alias,
                            alias_for: alias_for.clone()
                        };
                        if let AnyMetadata::Identifier { value } = alias.meta_data {
                            self.custom_types.insert(value.to_string(), TypedExpression::UserDefinedTypeAlias { identifier: value.to_string(), alias_for: Box::new(alias_for) });
                        };
                        return Statement::TypeDeclarationStatement(t)
                    } else {
                        panic!("Unexpected Type: {:?} expected a predefined type", self.lexer.peek());
                    }
                }
                TokenType::Struct => {
                    self.consume(TokenType::Struct);
                    self.consume(TokenType::Identifier);
                    let name_token = self.previous_token.expect("UNREACHABLE");
                    let name = if let AnyMetadata::Identifier { value } = name_token.meta_data {
                        value
                    } else {
                        panic!("Expected identifier");
                    };
                    self.consume(TokenType::LeftBrace);
                    let mut fields = vec![];
                    while !self.match_tokens(&[TokenType::RightBrace]) {
                        self.consume(TokenType::Identifier);
                        let field_name_token = self.previous_token.expect("UNREACHABLE");
                        let field_name = if let AnyMetadata::Identifier { value } = field_name_token.meta_data {
                            value
                        } else {
                            panic!("Expected identifier");
                        };
                        self.consume(TokenType::Colon);
                        let field_type = self.parse_type_expression();
                        fields.push(StructField {
                            name: field_name,
                            field_type
                        });
                        if self.match_tokens(&[TokenType::Comma]) {
                            continue;
                        }
                    }
                    self.custom_types.insert(name.to_string(), TypedExpression::Struct { name: name.to_string() });
                    return Statement::StructDeclaration(StructDeclaration {
                        name,
                        fields
                    });
                }

                _ => {
                    let expr = self.parse_expression();
                    if self.match_tokens(&[TokenType::Equal]) {
                        let rhs = self.parse_expression();
                        self.consume(TokenType::Semicolon);
                        if expr.is_lvalue() {
                            return Statement::VariableReassignmentStatement(VariableReassignmentStatement {
                                lhs: expr,
                                rhs
                            })
                        } else {
                            let position = self.previous_token.unwrap().position;
                            panic!("Trying to assign to something which can not be assigned, not an lvalue: {}:{}", position.line, position.column);
                        }
                    }
                    self.consume(TokenType::Semicolon);
                    return Statement::ExpressionStatement(ExpressionStatement {
                        value: expr,
                        position: starting_position
                    });
                }
            }
        }
        panic!("UNREACHABLE");
    }

    fn parse_type_expression(&mut self) -> TypedExpression {
        let current_token = self.lexer.next().unwrap();
        self.tt_to_typed(current_token)
    }

    fn consume(&mut self, tt: TokenType) {
        if let Some(Token { token_type, position, meta_data, .. }) = self.lexer.peek() {
            if *token_type == tt {
                self.previous_token = self.lexer.next();
            } else {
                panic!("Expected a {:?} found {:?} {:?}:{:?}", tt, *token_type, position.line, position.column);
            }
        }
    }

    fn parse_function(&mut self, starting_position: Position) -> Statement<'a> {
        let name = if self.match_tokens(&[TokenType::Identifier]) {
            if let Some(prev) = &self.previous_token {
                if let AnyMetadata::Identifier{ value } = &prev.meta_data {
                    *value
                } else {
                    panic!("Expected identifier metadata");
                }
            } else {
                panic!("No previous token");
            }
        } else {
            panic!("Expected function name identifier");
        };

        self.consume(TokenType::LeftParen);
        let mut args = Vec::new();

        while !self.match_tokens(&[TokenType::RightParen]) {
            args.push(self.parse_args());
        }

        let return_type = self.parse_type_expression();

        let body = self.parse_block_statement();
        if let Statement::BlockStatement(body) = body {
            Statement::FunctionDeclaration(FunctionDeclaration {
                name,
                arity: args.len(),
                arguments: args,
                body,
                return_type,
                position: starting_position,
                variable_size: 0
            })
        } else {
            panic!("UNREACHABLE");
        }

    }

    fn parse_block_statement(&mut self) -> Statement<'a> {
        self.consume(TokenType::LeftBrace);
        let mut stmts = vec![];
        let current_position = self.lexer.peek().unwrap().position.clone();
        while !self.match_tokens(&[TokenType::RightBrace]) {
            stmts.push(self.parse_statement());
        }

        Statement::BlockStatement(BlockStatement { values: stmts, position: current_position })
    }

    fn parse_args(&mut self) -> Argument<'a> {
        let arg_type = self.parse_type_expression();
        let previous_token = self.previous_token.clone().expect("UNREACHABLE");

        if self.match_tokens(&[TokenType::Identifier]) {
            let prev = self.previous_token.clone().expect("UNREACHABLE");
            if let AnyMetadata::Identifier { value: name } = &prev.meta_data {
                let _ = self.match_tokens(&[TokenType::Comma]);
                Argument {
                    name,
                    arg_type
                }
            } else {
                panic!("Expected identifier metadata");
            }
        } else {
            panic!("Expected an Identifier {}:{}", previous_token.position.line, previous_token.position.column);
        }

    }

    pub fn calculate_size_from_type(&self, t: &TypedExpression) -> usize {
        match t {
            TypedExpression::Integer => 4,
            TypedExpression::String => 8,
            TypedExpression::Float => 8,
            TypedExpression::Void => 1,
            TypedExpression::Pointer(_) => 8,
            TypedExpression::UserDefinedTypeAlias{ identifier: _, alias_for: u } => self.calculate_size_from_type(u),
            TypedExpression::Struct { .. } => 8,
            TypedExpression::Function { .. } => 8,
        }
    }

    pub fn calculate_variables_size(&self, bs: &BlockStatement<'a>) -> usize {
        let mut size = 8;
        for stmt in &bs.values {
            if let Statement::VarDeclaration(VarDeclarationStatement { variable_type, .. }) = stmt {
                size += self.calculate_size_from_type(variable_type);
            }
        }
        size
    }

    fn parse_expression(&mut self) -> Expression<'a> {
        self.equality()
    }

    fn equality(&mut self) -> Expression<'a> {
        self.create_binary_expr(vec![TokenType::BangEqual, TokenType::EqualEqual], Self::comparison)
    }

    fn comparison(&mut self) -> Expression<'a> {
        self.create_binary_expr(vec![TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual], Self::term)
    }

    fn term(&mut self) -> Expression<'a> {
        self.create_binary_expr(vec![TokenType::Minus, TokenType::Plus], Self::factor)
    }

    fn factor(&mut self) -> Expression<'a> {
        self.create_binary_expr(vec![TokenType::Slash, TokenType::Star], Self::unary)
    }
    
    fn unary(&mut self) -> Expression<'a> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus, TokenType::Ampersand, TokenType::Star]) {
            let operator = self.previous_token.expect("No Previous token given.");
            return Expression::Unary(UnaryExpression{
                operator,
                value: Box::from(self.unary())
            });
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Expression<'a> {
        let mut expr = self.primary();
        loop {
            if self.match_tokens(&[TokenType::LeftParen]) {
                let call_position = self.previous_token.expect("UNREACHABLE").position;
                let mut args = vec![];
                while let Some(t) = self.lexer.peek() {
                    if t.token_type == TokenType::RightParen {
                        break;
                    }
                    let arg = self.parse_expression();
                    args.push(arg);
                    if !self.match_tokens(&[TokenType::Comma]) {
                        continue;
                    }
                }
                let _ = self.lexer.next();
                expr = Expression::Call(CallExpression {
                    callee: Box::new(expr),
                    arguments: args,
                    position: call_position
                });
                continue;
            }
            if self.match_tokens(&[TokenType::Dot]) {
                self.consume(TokenType::Identifier);
                let field_token = self.previous_token.expect("UNREACHABLE");
                let field = if let AnyMetadata::Identifier { value } = field_token.meta_data {
                    value
                } else {
                    panic!("Expected identifier after '.'");
                };
                expr = Expression::FieldAccess(FieldAccessExpression {
                    target: Box::new(expr),
                    field,
                    position: field_token.position
                });
                continue;
            }
            break;
        }
        expr
    }

    fn primary(&mut self) -> Expression<'a> {
        if let Some(token) = self.lexer.peek() {
            self.previous_token = Some(*token);
            let tok = self.lexer.next().expect("UNREACHABLE");
            let pos = tok.position;
            if tok.token_type == TokenType::Pound {
                self.consume(TokenType::Identifier);
                let struct_name_token = self.previous_token.expect("UNREACHABLE");
                let struct_name = if let AnyMetadata::Identifier { value } = struct_name_token.meta_data {
                    value
                } else {
                    panic!("Expected struct name after '#'");
                };
                self.consume(TokenType::LeftBrace);
                let mut fields = vec![];
                while !self.match_tokens(&[TokenType::RightBrace]) {
                    self.consume(TokenType::Identifier);
                    let field_name_token = self.previous_token.expect("UNREACHABLE");
                    let field_name = if let AnyMetadata::Identifier { value } = field_name_token.meta_data {
                        value
                    } else {
                        panic!("Expected field name");
                    };
                    self.consume(TokenType::Colon);
                    let field_value = self.parse_expression();
                    fields.push(StructLiteralField {
                        name: field_name,
                        value: field_value
                    });
                    if self.match_tokens(&[TokenType::Comma]) {
                        continue;
                    }
                }
                return Expression::StructLiteral(StructLiteralExpression {
                    name: struct_name,
                    fields,
                    position: pos
                });
            }
            if ([TokenType::Integer, TokenType::String, TokenType::Identifier, TokenType::String, TokenType::Void]).contains(&tok.token_type) {
                Expression::Literal(LiteralExpression {
                    value: tok
                })
            } else {
                panic!("INVALID Primary Type: {:?} {}:{} ", tok, pos.line, pos.column);
            }
        } else {
            panic!("UNREACHABLE");
        }
    }

    fn create_binary_expr(
        &mut self,
        match_tokens: Vec<TokenType>,
        precedent_function: fn(&mut Self) -> Expression<'a>,
    ) -> Expression<'a> {
        let mut expr = precedent_function(self);
        while self.match_tokens(&match_tokens) {
            let operator = self.previous_token.expect("Token must exist here");
            let right_expression = precedent_function(self);

            expr = Expression::Binary(BinaryExpression {
                left: Box::new(expr),
                operator,
                right: Box::new(right_expression),
            });
        }
        expr
    }

    fn match_tokens(&mut self, to_match: &[TokenType]) -> bool {
        if let Some(tok) = self.lexer.peek() && to_match.contains(&tok.token_type) {
            self.previous_token = self.lexer.next();
            return true;
        }
        false
    }
}
