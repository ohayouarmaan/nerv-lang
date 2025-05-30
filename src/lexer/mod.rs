use crate::shared::{
    errors::LexerError, meta::{AnyMetadata, NumberType}, positions::Position, tokens::{
        Token,
        TokenType
    }
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Lexer<'a> {
    source_code: &'a str,
    position: usize,
    current_line: usize,
    current_column: usize,
}


#[allow(dead_code)]
impl<'a> Lexer<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self {
            source_code,
            position: 0,
            current_line: 1,
            current_column: 0
        }
    }

    fn can_move(&self) -> bool {
        if self.position < self.source_code.len() {
            return true;
        }
        false
    }

    fn advance(&mut self) -> Result<(), LexerError> {
        if self.can_move() {
            self.position += 1;
            self.current_column += 1;
            return Ok(())
        }
        Err(LexerError::CalledNextAfterExhaustion)
    }

    fn generate_operator(&mut self, lexeme_start: usize, tt: TokenType) -> Option<Token<AnyMetadata<'a>>> {
        self.advance().ok()?;
        let lexeme_ending = self.position;
        Some(Token {
            token_type: tt,
            position: Position::new(self.current_line, self.current_column),
            lexeme: (lexeme_start, lexeme_ending),
            meta_data: AnyMetadata::None
        })
    }

    fn get_current_character(&self) -> Result<char, LexerError> {
        self.source_code.chars().nth(self.position).ok_or(LexerError::IllegalCharacterAccess)
    }

    fn generate_string(&mut self, lexeme_start: usize) -> Result<Token<AnyMetadata<'a>>, LexerError> {
        self.advance().map_err(|_| LexerError::UnexpectedEof)?;
        let starting_column = self.current_column;
        while self.can_move() && self.get_current_character().map_err(|_| LexerError::UnexpectedEof)? != '"' {
            self.advance().map_err(|_| LexerError::UnexpectedEof)?;
        }
        self.advance().map_err(|_| LexerError::UnexpectedEof)?;
        let lexeme_end = self.position;
        Ok(Token {
            token_type: TokenType::String,
            position: Position::new(self.current_line, starting_column),
            lexeme: (lexeme_start, lexeme_end),
            meta_data: AnyMetadata::String {
                value: &self.source_code[lexeme_start..lexeme_end]
            }
        })
    }

    fn get_keyword_type(&self, lexeme_start: usize, lexeme_end: usize) -> Result<TokenType, LexerError> {
        match &self.source_code[lexeme_start..lexeme_end] {
            "and" => Ok(TokenType::And),
            "else" => Ok(TokenType::Else),
            "false" => Ok(TokenType::False),
            "fun" => Ok(TokenType::Fun),
            "for" => Ok(TokenType::For),
            "if" => Ok(TokenType::If),
            "nil" => Ok(TokenType::Nil),
            "or" => Ok(TokenType::Or),
            "print" => Ok(TokenType::Print),
            "return" => Ok(TokenType::Return),
            "super" => Ok(TokenType::Super),
            "this" => Ok(TokenType::This),
            "true" => Ok(TokenType::True),
            "var" => Ok(TokenType::Var),
            "dec" => Ok(TokenType::Dec),
            "while" => Ok(TokenType::While),
            "int" => Ok(TokenType::DInteger),
            "string" => Ok(TokenType::DString),
            "char" => Ok(TokenType::DChar),
            "float" => Ok(TokenType::DFloat),
            "void" => Ok(TokenType::DVoid),
            "extern" => Ok(TokenType::Extern),
            "unit" => Ok(TokenType::Void),
            _ => Err(LexerError::IllegalKeyword),
        }
    }

    fn generate_keyword(&mut self, lexeme_start: usize) -> Result<Token<AnyMetadata<'a>>, LexerError> {
        self.advance().map_err(|_| LexerError::UnexpectedEof)?;
        let starting_column = self.current_column;
        while self.can_move(){
            let ch = self.get_current_character().map_err(|_| LexerError::UnexpectedEof)?;
            if !ch.is_ascii_alphabetic() {
                break
            }
            self.advance().map_err(|_| LexerError::UnexpectedEof)?;
        }
        let lexeme_end = self.position;
        match self.get_keyword_type(lexeme_start, lexeme_end) {
            Ok(tt) => {
                Ok(Token {
                    token_type: tt,
                    position: Position::new(self.current_line, starting_column),
                    lexeme: (lexeme_start, lexeme_end),
                    meta_data: AnyMetadata::None
                })
            }
            Err(LexerError::IllegalKeyword) => {
                Ok(Token {
                    token_type: TokenType::Identifier,
                    position: Position::new(self.current_line, starting_column),
                    lexeme: (lexeme_start, lexeme_end),
                    meta_data: AnyMetadata::Identifier {
                        value: &self.source_code[lexeme_start..lexeme_end]
                    }
                })
            }
            _ => {
                panic!("Something wen't wrong while lexing.");
            }
        }
    }

    fn generate_number(&mut self, lexeme_start: usize) -> Result<Token<AnyMetadata<'a>>, LexerError> {
        let starting_column = self.current_column;
        let mut dot_count = 0;
        while self.can_move() {
            if let Ok(c) = self.get_current_character() {
                if c.is_ascii_digit() || c == '_' || c == '.' {
                    if c == '.'{
                        dot_count += 1;
                        if dot_count > 1 {
                            break;
                        }
                    }
                   self.advance().map_err(|_| LexerError::UnexpectedEof)?;
                } else {
                    break;
                }
            }
        }
        if dot_count == 1 {
            let value = match self.source_code[lexeme_start..self.position].parse::<f64>() {
                Ok(x) => x,
                Err(_) => {
                    return Err(LexerError::IllegalNumber);
                }
            };
            Ok(Token {
                token_type: TokenType::Integer,
                position: Position::new(self.current_line, starting_column),
                lexeme: (lexeme_start, self.position),
                meta_data: AnyMetadata::Number {
                    value: NumberType::Float(value),
                }
            })
        } else {
            let value = match self.source_code[lexeme_start..self.position].parse::<i64>() {
                Ok(x) => x,
                Err(_) => {
                    return Err(LexerError::IllegalNumber);
                }
            };
            Ok(Token {
                token_type: TokenType::Integer,
                position: Position::new(self.current_line, starting_column),
                lexeme: (lexeme_start, self.position),
                meta_data: AnyMetadata::Number {
                    value: NumberType::Integer(value),
                }
            })
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<AnyMetadata<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut lexeme_start = self.position;
        while self.can_move() {
            if let Some(ch) = self.source_code.chars().nth(self.position) {
                match ch {
                    '\n' => {
                        self.current_line += 1;
                        self.current_column = 0;
                        self.advance().ok()?
                    }

                    // Operators
                    '+' => return self.generate_operator(lexeme_start, TokenType::Plus),
                    '-' => return self.generate_operator(lexeme_start, TokenType::Minus),
                    '/' => return self.generate_operator(lexeme_start, TokenType::Slash),
                    '*' => return self.generate_operator(lexeme_start, TokenType::Star),
                    '@' => return self.generate_operator(lexeme_start, TokenType::At),
                    '#' => return self.generate_operator(lexeme_start, TokenType::Pound),

                    // Words
                    'a' ..= 'z' | 'A' ..= 'Z' | '_' => return self.generate_keyword(lexeme_start).ok(),
                    '"' => return self.generate_string(lexeme_start).ok(),
                    '0' ..= '9' => return self.generate_number(lexeme_start).ok(),

                    // Eof
                    '\0' => {
                        self.current_column += 1;
                        return None;
                    },
                    ';' => {
                        self.advance().ok()?;
                        return Some(Token {
                            token_type: TokenType::Semicolon,
                            position: Position::new(self.current_line, self.current_column),
                            lexeme: (lexeme_start, self.position),
                            meta_data: AnyMetadata::None
                        })
                    }

                    '=' => {
                        self.advance().ok()?;
                        return Some(Token {
                            token_type: TokenType::Equal,
                            position: Position::new(self.current_line, self.current_column),
                            lexeme: (lexeme_start, self.position),
                            meta_data: AnyMetadata::None
                        })
                    }

                    '(' => {
                        self.advance().ok()?;
                        return Some(Token {
                            token_type: TokenType::LeftParen,
                            position: Position::new(self.current_line, self.current_column),
                            lexeme: (lexeme_start, self.position),
                            meta_data: AnyMetadata::None
                        })
                    }

                    ')' => {
                        self.advance().ok()?;
                        return Some(Token {
                            token_type: TokenType::RightParen,
                            position: Position::new(self.current_line, self.current_column),
                            lexeme: (lexeme_start, self.position),
                            meta_data: AnyMetadata::None
                        })
                    }

                    '{' => {
                        self.advance().ok()?;
                        return Some(Token {
                            token_type: TokenType::LeftBrace,
                            position: Position::new(self.current_line, self.current_column),
                            lexeme: (lexeme_start, self.position),
                            meta_data: AnyMetadata::None
                        })
                    }

                    '}' => {
                        self.advance().ok()?;
                        return Some(Token {
                            token_type: TokenType::RightBrace,
                            position: Position::new(self.current_line, self.current_column),
                            lexeme: (lexeme_start, self.position),
                            meta_data: AnyMetadata::None
                        })
                    }

                    x => {
                        if [' ', '\t'].contains(&x) {
                            self.advance().ok();
                            lexeme_start = self.position;
                            continue;
                        } else {
                            panic!("ILLEGAL CHARACTER: {:?}:{:?}", self.current_line, self.current_column);
                        }
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn lexing_operators() {
    //     let test = "5 + 4\0";
    //     let mut lex = Lexer::new(test);
    //     if let Some(token1) = lex.next() {
    //         assert_eq!(token1.token_type, TokenType::Integer);
    //     }
    //     if let Some(token2) = lex.next() {
    //         assert_eq!(token2.token_type, TokenType::Plus);
    //     }
    //     if let Some(token3) = lex.next() {
    //         assert_eq!(token3.token_type, TokenType::Integer);
    //     }
    // }
    //
    // #[test]
    // fn lexing_strings() {
    //     let test = "\"Hello, World!\"\0";
    //     let mut lex = Lexer::new(test);
    //     if let Some(string_token) = lex.next() {
    //         assert_eq!(string_token.token_type, TokenType::String);
    //         assert_eq!(&test[string_token.lexeme.0..string_token.lexeme.1], "\"Hello, World!\"");
    //         assert_eq!(string_token.position.line, 1);
    //         assert_eq!(string_token.position.column, 1);
    //     }
    //     if let Some(eof_token) = lex.next() {
    //         assert_eq!(eof_token.token_type, TokenType::Eof);
    //         assert_eq!(eof_token.position.line, 1);
    //         assert_eq!(eof_token.position.column, 16);
    //     }
    // }

    // #[test]
    // fn lexing_numbers() {
    //     let test = "12_00_00_000\0";
    //     let mut lex = Lexer::new(test);
    //     if let Some(number_token) = lex.next() {
    //         assert_eq!(number_token.token_type, TokenType::Integer);
    //         assert_eq!(&test[number_token.lexeme.0..number_token.lexeme.1], "12_00_00_000");
    //         assert_eq!(number_token.position.line, 1);
    //         assert_eq!(number_token.position.column, 0);
    //     }
    //     if let Some(eof_token) = lex.next() {
    //         assert_eq!(eof_token.token_type, TokenType::Eof);
    //         assert_eq!(eof_token.position.line, 1);
    //         assert_eq!(eof_token.position.column, 13);
    //     }
    // }
}
