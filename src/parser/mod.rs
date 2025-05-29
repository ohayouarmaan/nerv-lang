use crate::{
    lexer::Lexer,
    shared::{
        meta::AnyMetadata,
        parser_nodes::{
            BinaryExpression, Expression, ExpressionStatement, LiteralExpression, Program, Statement, UnaryExpression, VarDeclarationStatement
        },
        tokens::{
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
                                    variable_type: data_type
                                });
                            } else {
                                println!("UNEXPECTED: {:?}", after_data_type);
                            }
                        }
                    }
                }
                _ => {
                    let expr = self.parse_expression();
                    dbg!(&expr);
                    self.consume(TokenType::Semicolon);
                    return Statement::ExpressionStatement(ExpressionStatement {
                        value: expr
                    });
                }
            }
        }
        panic!("UNREACHABLE");
    }

    fn consume(&mut self, tt: TokenType) {
        if let Some(Token { token_type, .. }) = self.lexer.peek() {
            if *token_type == tt {
                self.previous_token = self.lexer.next();
            } else {
                panic!("Expected a {:?} found {:?}", tt, *token_type);
            }
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
            if  ([TokenType::Integer, TokenType::String, TokenType::Identifier, TokenType::String]).contains(&tok.token_type) {
                let e = Expression::Literal(LiteralExpression {
                    value: tok
                });
                e
            } else {
                panic!("INVALID");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{meta::{AnyMetadata, NumberType}, tokens::TokenType};
    
    #[test]
    fn check_parsing_expression() {
        let source_code = "5 + 4 * 0;\0";
        let mut parser = Parser::new(source_code);
        let program = parser.parse();

        assert_eq!(program.stmts.len(), 1);
        match &program.stmts[0] {
            Statement::ExpressionStatement(ExpressionStatement { value: Expression::Binary(add_expr) }) => {
                match *add_expr.left {
                    Expression::Literal(ref lit) => {
                        match lit.value.meta_data {
                                    AnyMetadata::Number { value: NumberType::Integer(n) } => assert_eq!(n, 5),
                            _ => panic!("Expected number metadata on left of '+'"),
                        }
                    },
                    _ => panic!("Expected literal on left of '+'"),
                }

                assert_eq!(add_expr.operator.token_type, TokenType::Plus);

                match *add_expr.right {
                    Expression::Binary(ref mul_expr) => {
                        match *mul_expr.left {
                            Expression::Literal(ref lit) => {
                                match lit.value.meta_data {
                                    AnyMetadata::Number { value: NumberType::Integer(n) } => assert_eq!(n, 4),
                                    _ => panic!("Expected number metadata on left of '*'"),
                                }
                            },
                            _ => panic!("Expected literal on left of '*'"),
                        }

                        assert_eq!(mul_expr.operator.token_type, TokenType::Star);

                        match *mul_expr.right {
                            Expression::Literal(ref lit) => {
                                match lit.value.meta_data {
                                    AnyMetadata::Number { value: NumberType::Integer(n) } => assert_eq!(n, 0),
                                    _ => panic!("Expected number metadata on right of '*'"),
                                }
                            },
                            _ => panic!("Expected literal on right of '*'"),
                        }
                    },
                    _ => panic!("Expected binary '*' expression on right of '+'"),
                }
            },
            _ => panic!("Expected binary '+' expression at root"),
        }
    }

    #[test]
    fn check_variable() {
        let source_code = "dec fifteen int = 5 * 3;\n 5 + fifteen;\0";
        let mut parser = Parser::new(source_code);
        let program = parser.parse();
        dbg!(program);
        assert_eq!(true, true)
    }
}
