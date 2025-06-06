use super::{meta::AnyMetadata, positions::Position, tokens::{ Token, TokenType }};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Expression<'a> {
    Binary(BinaryExpression<'a>),
    Unary(UnaryExpression<'a>),
    Literal(LiteralExpression<'a>),
    Call(CallExpression<'a>)
}

impl Expression<'_> {
    pub fn is_lvalue(&self) -> bool {
        matches!(self, Self::Literal(LiteralExpression{ value: Token{ meta_data: AnyMetadata::Identifier{..}, .. }, .. }) 
                | Self::Unary(UnaryExpression { operator: Token { token_type: TokenType::Star, ..}, .. }))
    }
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
pub struct CallExpression<'a> {
    pub name: &'a str,
    pub arguments: Vec<Expression<'a>>,
    pub position: Position
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
    ExpressionStatement(ExpressionStatement<'a>),
    FunctionDeclaration(FunctionDeclaration<'a>),
    BlockStatement(BlockStatement<'a>),
    ReturnStatement(ReturnStatement<'a>),
    ExternStatement(ExternFunctionStatement<'a>),
    VariableReassignmentStatement(VariableReassignmentStatement<'a>),
    TypeDeclarationStatement(TypeDeclarationStatement<'a>)
}

#[derive(Debug, Clone)]
pub struct TypeDeclarationStatement<'a> {
    pub alias: Token<AnyMetadata<'a>>,
    pub alias_for: TypedExpression
}

#[derive(Debug, Clone)]
pub struct VariableReassignmentStatement<'a> {
    pub lhs: Expression<'a>,
    pub rhs: Expression<'a>
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Argument<'a> {
    pub name: &'a str,
    pub arg_type: TypedExpression
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct FunctionDeclaration<'a> {
    pub name: &'a str,
    pub arity: usize,
    pub arguments: Vec<Argument<'a>>,
    pub body: BlockStatement<'a>,
    pub return_type: TypedExpression,
    pub position: Position,
    pub variable_size: usize
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct VarDeclarationStatement<'a> {
    pub name: &'a str,
    pub value: Expression<'a>,
    pub variable_type: TypedExpression,
    pub position: Position
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExpressionStatement<'a> {
    pub value: Expression<'a>,
    pub position: Position
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BlockStatement<'a> {
    pub values: Vec<Statement<'a>>,
    pub position: Position
}


#[derive(Debug, Clone)]
pub struct ReturnStatement<'a> {
    pub value: Expression<'a>,
    pub position: Position
}

#[derive(Debug, Clone)]
pub struct FunctionSignatureDeclaration<'a> {
    pub fx_name: &'a str,
    pub args: Vec<TypedExpression>,
    pub return_type: TypedExpression
}

#[derive(Debug, Clone)]
pub struct ExternFunctionStatement<'a> {
    pub fx_name: &'a str,
    pub fx_sig: FunctionSignatureDeclaration<'a> 
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypedExpression {
    Integer,
    String,
    Float,
    Void,
    Pointer(Box<TypedExpression>),
    UserDefinedTypeAlias {
        identifier: String,
        alias_for: Box<TypedExpression>
    }
}


