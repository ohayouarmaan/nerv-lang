use crate::{
    lexer::Lexer,
    shared::{
        meta::AnyMetadata, parser_nodes::{
            Argument, BinaryExpression, BlockStatement, CallExpression, Expression, ExpressionStatement, FunctionDeclaration, LiteralExpression, Program, ReturnStatement, Statement, UnaryExpression, VarDeclarationStatement
        }, positions::Position, tokens::{
            Token,
            TokenType
        }
    }
};
use std::iter::Peekable;

#[allow(dead_code)]
pub struct Parser<'a> {
    pub lexer: Peekable<Lexer<'a>>,
    pub previous_token: Option<Token<AnyMetadata<'a>>>
}


#[allow(dead_code)]
impl<'a> Parser<'a> {
    pub fn new(source_code: &'a str) -> Self {
        let lexer = Lexer::new(source_code).peekable();
        Self {
            lexer,
            previous_token: None
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
                            let data_type: TokenType;
                            let c_token = self.lexer.next();
                            if let Some(Token { token_type, .. }) = c_token{
                                data_type = token_type;
                            } else {
                                panic!("Invalid data type. {:?}", c_token);
                            } 
                            let after_data_type = self.lexer.next();
                            println!("WE'RE HERE: {:?}", data_type);
                            if let Some(Token { token_type: TokenType::Equal, .. }) = after_data_type {
                                let expr = self.parse_expression();
                                self.consume(TokenType::Semicolon);
                                dbg!(&expr, &value, &data_type);
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
                _ => {
                    let expr = self.parse_expression();
                    dbg!(&expr);
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

    fn consume(&mut self, tt: TokenType) {
        if let Some(Token { token_type, position, .. }) = self.lexer.peek() {
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

        let return_type: TokenType = if self.match_tokens(&[TokenType::DInteger, TokenType::DString, TokenType::DFloat, TokenType::DVoid]) {
            self.previous_token.clone().unwrap().token_type
        } else {
            let error_position = self.lexer.peek().unwrap().position.clone();
            panic!("Unknown return type: {}:{}", error_position.line, error_position.column);
        };

        let body = self.parse_block_statement();

        if let Statement::BlockStatement(body) = body {
            Statement::FunctionDeclaration(FunctionDeclaration {
                name,
                arity: args.len(),
                arguments: args,
                body,
                return_type,
                position: starting_position
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
        let matched = self.match_tokens(&[TokenType::DInteger, TokenType::DString]);

        if matched {
            let arg_type = self.previous_token.clone().expect("UNREACHABLE");

            if self.match_tokens(&[TokenType::Identifier]) {
                let prev = self.previous_token.clone().expect("UNREACHABLE");
                if let AnyMetadata::Identifier { value: name } = &prev.meta_data {
                    Argument {
                        name,
                        arg_type: arg_type.token_type,
                    }
                } else {
                    panic!("Expected identifier metadata");
                }
            } else {
                panic!();
            }
        } else {
            panic!();
        }
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
        if self.match_tokens(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous_token.clone().expect("No Previous token given.");
            return Expression::Unary(UnaryExpression{
                operator,
                value: Box::from(self.unary())
            })
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
