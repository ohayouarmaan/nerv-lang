use crate::{
    lexer::Lexer,
    shared::{
        meta::{
            AnyMetadata,
            NumberMetaData
        },
        positions::Position,
        parser_nodes::{
            BinaryExpression,
            Expression,
            LiteralExpression,
            Program,
            UnaryExpression
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
        let mut exprs: Vec<Expression> = vec![];
        while self.lexer.peek().is_some() {
            exprs.push(self.parse_expression());
        }

        Program {
            stmts: exprs
        }
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
            let e = Expression::Literal(LiteralExpression {
                value: self.lexer.next().expect("UNREACHABLE")
            });
            e
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
    use crate::shared::{meta::AnyMetadata, tokens::TokenType};
    
    #[test]
    fn check_parsing_expression() {
        let source_code = "5 + 4\0";
        let mut parser = Parser::new(source_code);
        let program = parser.parse();

        assert_eq!(program.stmts.len(), 1);

        match &program.stmts[0] {
            Expression::Binary(binary_expr) => {
                match *binary_expr.left {
                    Expression::Literal(ref lit) => {
                        match lit.value.meta_data {
                            AnyMetadata::Number(num) => assert_eq!(num.value, 5),
                            _ => panic!("Expected number metadata on left"),
                        }
                    },
                    _ => panic!("Expected literal on left"),
                }

                // Check operator is '+'
                assert_eq!(binary_expr.operator.token_type, TokenType::Plus);

                // Check right is a literal "4"
                match *binary_expr.right {
                    Expression::Literal(ref lit) => {
                        match lit.value.meta_data {
                            AnyMetadata::Number(num) => assert_eq!(num.value, 4),
                            _ => panic!("Expected number metadata on right"),
                        }
                    },
                    _ => panic!("Expected literal on right"),
                }
            },
            _ => panic!("Expected binary expression at root"),
        }
    }
}
