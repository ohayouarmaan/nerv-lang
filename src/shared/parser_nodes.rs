use super::{meta::AnyMetadata, tokens::{ Token, TokenType }};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Binary(BinaryExpression<'a>),
    Unary(UnaryExpression<'a>),
    Literal(LiteralExpression<'a>)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BinaryExpression<'a> {
    pub left: Box<Expression<'a>>,
    pub operator: Token<AnyMetadata<'a>>,
    pub right: Box<Expression<'a>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct UnaryExpression<'a> {
    pub operator: Token<AnyMetadata<'a>>,
    pub value: Box<Expression<'a>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LiteralExpression<'a> {
    pub value: Token<AnyMetadata<'a>>
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Program<'a> {
    //TODO: until we reach statements this will hold expressions, but after that it shall hold
    //statements
    pub stmts: Vec<Statement<'a>>
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Statement<'a> {
    VarDeclaration(VarDeclarationStatement<'a>),
    ExpressionStatement(ExpressionStatement<'a>)
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VarDeclarationStatement<'a> {
    pub name: &'a str,
    pub value: Expression<'a>,
    pub variable_type: TokenType
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionStatement<'a> {
    pub value: Expression<'a>,
}

