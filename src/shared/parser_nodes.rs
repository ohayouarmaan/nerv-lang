use super::{meta::AnyMetadata, tokens::Token};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Expression<'a> {
    Binary(BinaryExpression<'a>),
    Unary(UnaryExpression<'a>),
    Literal(LiteralExpression<'a>)
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct BinaryExpression<'a> {
    pub left: Box<Expression<'a>>,
    pub operator: Token<AnyMetadata<'a>>,
    pub right: Box<Expression<'a>>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct UnaryExpression<'a> {
    pub operator: Token<AnyMetadata<'a>>,
    pub value: Box<Expression<'a>>,
}


#[allow(dead_code)]
#[derive(Debug)]
pub struct LiteralExpression<'a> {
    pub value: Token<AnyMetadata<'a>>
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Program<'a> {
    //TODO: until we reach statements this will hold expressions, but after that it shall hold
    //statements
    pub stmts: Vec<Expression<'a>>
}
