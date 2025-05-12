use crate::shared::positions::Position;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    At,
    Pound,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    // End of file.
    Eof,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Token<T: Clone> {
    pub token_type: TokenType,
    pub position: Position,
    pub lexeme: (usize, usize),
    pub meta_data: T,
}
