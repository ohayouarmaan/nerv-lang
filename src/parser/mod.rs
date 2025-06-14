use crate::{
    lexer::Lexer,
    shared::{
        meta::AnyMetadata, parser_nodes::{
            Argument, BinaryExpression, BlockStatement, CallExpression, Expression, ExpressionStatement, ExternFunctionStatement, FunctionDeclaration, FunctionSignatureDeclaration, LiteralExpression, Program, ReturnStatement, Statement, StructDefinition, StructExpression, StructItem, StructItemExpression, TypeDeclarationStatement, TypedExpression, ParserTypedStructField, UnaryExpression, VarDeclarationStatement, VariableReassignmentStatement
        }, positions::Position, tokens::{
            Token,
            TokenType
        }
    }
};
use core::panic;
use std::{any::Any, collections::HashMap, iter::Peekable};

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
            }
            _ => {
                panic!("");
            }
        }
    }

    fn get_name_from_identifier(&self, t: Token<AnyMetadata<'a>>) -> &'a str {
        if let Token { token_type: TokenType::Identifier, meta_data: AnyMetadata::Identifier { value: name }, .. } = t {
            return name;
        }
        panic!("Not an identifier");
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
            let starting_position = current_token.position.clone();
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
                TokenType::Struct => {
                    self.consume(TokenType::Struct);
                    self.consume(TokenType::Identifier);
                    let struct_name: &'a str = self.get_name_from_identifier(self.previous_token.unwrap());

                    let mut fields: Vec<StructItem> = Vec::new();
                    let mut typed_fields: Vec<ParserTypedStructField> = Vec::new();
                    self.consume(TokenType::LeftBrace);
                    while !self.match_tokens(&[TokenType::RightBrace]) {
                        self.consume(TokenType::Identifier);
                        let field_name = self.get_name_from_identifier(self.previous_token.unwrap());
                        let field_type = self.parse_type_expression();
                        let mut field_value: Option<Expression> = None;
                        if self.match_tokens(&[TokenType::Equal]) {
                            field_value = Some(self.parse_expression());
                        }
                        let copied_field_type = field_type.clone();
                        fields.push(StructItem { name: field_name, item_type: field_type, value: field_value });
                        typed_fields.push(ParserTypedStructField { field_type: Box::new(copied_field_type), field_name: field_name.to_string() });
                        self.match_tokens(&[TokenType::Comma]);
                    }
                    self.custom_types.insert(struct_name.to_string(), TypedExpression::Struct { fields: typed_fields });
                    return Statement::StructDeclarationStatement(StructDefinition {
                        name: struct_name,
                        fields
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
                dbg!(token_type, meta_data);
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
        println!("name: {:?}", name);

        self.consume(TokenType::LeftParen);
        let mut args = Vec::new();

        while !self.match_tokens(&[TokenType::RightParen]) {
            args.push(self.parse_args());
        }

        let return_type = self.parse_type_expression();

        let body = self.parse_block_statement();
        let var_size: usize =if let Statement::BlockStatement(bs) = &body {
            self.calculate_variables_size(bs)
        } else {
            unreachable!()
        };

        if let Statement::BlockStatement(body) = body {
            Statement::FunctionDeclaration(FunctionDeclaration {
                name,
                arity: args.len(),
                arguments: args,
                body,
                return_type,
                position: starting_position,
                variable_size: var_size
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
                dbg!(&name, &arg_type);
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
            TypedExpression::Struct { fields } => {
                let mut size = 0;
                for field in fields {
                    size += self.calculate_size_from_type(&field.field_type)
                }
                size
            },
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
        let t = self.equality();
        t
    }

    fn equality(&mut self) -> Expression<'a> {
        let e = self.create_binary_expr(vec![TokenType::BangEqual, TokenType::EqualEqual], Self::comparison);
        e
    }

    fn comparison(&mut self) -> Expression<'a> {
        let x = self.create_binary_expr(vec![TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual], Self::term);
        x
    }

    fn term(&mut self) -> Expression<'a> {
        let x = self.create_binary_expr(vec![TokenType::Minus, TokenType::Plus], Self::factor);
        x
    }

    fn factor(&mut self) -> Expression<'a> {
        let t = self.create_binary_expr(vec![TokenType::Slash, TokenType::Star], Self::unary);
        t
    }
    
    fn unary(&mut self) -> Expression<'a> {
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus, TokenType::Ampersand, TokenType::Star]) {
            let operator = self.previous_token.clone().expect("No Previous token given.");
            return Expression::Unary(UnaryExpression{
                operator,
                value: Box::from(self.unary())
            });
        }
        let t = self.primary();
        t
    }

    fn primary(&mut self) -> Expression<'a> {
        if let Some(token) = self.lexer.peek() {
            self.previous_token = Some(token.clone());
            let tok = self.lexer.next().expect("UNREACHABLE");
            let pos = tok.position.clone();
            if  ([TokenType::Integer, TokenType::String, TokenType::Identifier, TokenType::String, TokenType::Void]).contains(&tok.token_type) {
                if let Some(next_token) = self.lexer.peek() {
                    if next_token.token_type == TokenType::LeftParen {
                        let prev = self.previous_token.clone().unwrap();
                        let _ = self.lexer.next();
                        let (prev_name, pos) = if let AnyMetadata::Identifier{ value } = prev.meta_data {
                            (value, prev.position)
                        } else {
                            panic!();
                        };
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
                        let e = Expression::Call(CallExpression {
                            name: prev_name,
                            arguments: args,
                            position: pos
                        });
                        return e;
                    }
                }
                let e = Expression::Literal(LiteralExpression {
                    value: tok
                });
                e
            } else if tok.token_type == TokenType::LeftBrace {
                let mut fields: Vec<StructItemExpression<'a>> = Vec::new();
                while !self.match_tokens(&[TokenType::RightBrace]) {
                    self.consume(TokenType::Identifier);
                    let name = self.get_name_from_identifier(self.previous_token.unwrap());
                    self.consume(TokenType::Colon);
                    let field_value = self.parse_expression();
                    fields.push(StructItemExpression { field_name: name, field_value: field_value });
                    self.match_tokens(&[TokenType::Comma]);
                }
                return Expression::Struct(StructExpression {
                    fields
                });
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
            let operator = self.previous_token.clone().expect("Token must exist here");
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
        if let Some(tok) = self.lexer.peek() {
            if to_match.contains(&tok.token_type) {
                self.previous_token = self.lexer.next();
                return true;
            }
        }
        false
    }
}
