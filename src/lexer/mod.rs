use crate::shared::{
    errors::LexerError, meta::{AnyMetadata, NumberMetaData, StringMetadata}, positions::Position, tokens::{
        Token,
        TokenType
    }
};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Lexer<'a> {
    source_code: &'a str,
    current_index: usize,
    current_line: usize,
    current_column: usize,
}


#[allow(dead_code)]
impl<'a> Lexer<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self {
            source_code,
            current_index: 0,
            current_line: 1,
            current_column: 0
        }
    }

    fn can_move(&self) -> bool {
        if self.current_index < self.source_code.len() {
            return true;
        }
        false
    }

    fn advance(&mut self) -> Result<(), LexerError> {
        if self.can_move() {
            self.current_index += 1;
            self.current_column += 1;
            return Ok(())
        }
        Err(LexerError::CalledNextAfterExhaustion)
    }

    fn generate_operator(&mut self, lexeme_start: usize, tt: TokenType) -> Option<Token<AnyMetadata<'a>>> {
        self.advance().ok()?;
        let lexeme_ending = self.current_index;
        Some(Token {
            token_type: tt,
            position: Position::new(self.current_line, self.current_column),
            lexeme: (lexeme_start, lexeme_ending),
            meta_data: AnyMetadata::None
        })
    }

    fn get_current_character(&self) -> Result<char, LexerError> {
        self.source_code.chars().nth(self.current_index).ok_or(LexerError::IllegalCharacterAccess)
    }

    fn generate_string(&mut self, lexeme_start: usize) -> Option<Token<AnyMetadata<'a>>> {
        self.advance().ok()?;
        let starting_column = self.current_column;
        while self.can_move() && self.get_current_character().ok()? != '"' {
            self.advance().ok()?;
        }
        self.advance().ok()?;
        let lexeme_end = self.current_index;
        Some(Token {
            token_type: TokenType::String,
            position: Position::new(self.current_line, starting_column),
            lexeme: (lexeme_start, lexeme_end),
            meta_data: AnyMetadata::String(StringMetadata {
                value: &self.source_code[lexeme_start..lexeme_end]
            })
        })
    }

    fn generate_number(&mut self, lexeme_start: usize) -> Result<Token<AnyMetadata<'a>>, LexerError> {
        let starting_column = self.current_column;
        while self.can_move() {
            if let Ok(c) = self.get_current_character() {
                if c.is_ascii_digit() || c == '_' {
                   self.advance().map_err(|_| LexerError::UnexpectedEof)?;
                } else {
                    break;
                }
            }
        }
        let value = match self.source_code[lexeme_start..self.current_column].parse::<u64>() {
            Ok(x) => x,
            Err(_) => {
                return Err(LexerError::IllegalNumber);
            }
        };
        Ok(Token {
            token_type: TokenType::Number,
            position: Position::new(self.current_line, starting_column),
            lexeme: (lexeme_start, self.current_index),
            meta_data: AnyMetadata::Number(NumberMetaData {
                value
            })
        })
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token<AnyMetadata<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let lexeme_start = self.current_index;
        while self.can_move() {
            if let Some(ch) = self.source_code.chars().nth(self.current_index) {
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

                    'a' ..= 'z' | 'A' ..= 'Z' | '_' => {
                        todo!("Create keyword here.");
                    }
                    '"' => return self.generate_string(lexeme_start),
                    '0' ..= '9' => return self.generate_number(lexeme_start).ok(),
                    '\0' => {
                        self.current_column += 1;
                        return Some(Token{
                            token_type: TokenType::Eof,
                            position: Position::new(
                                self.current_line,
                                self.current_column
                            ),
                            lexeme: (self.current_index, self.current_index),
                            meta_data: AnyMetadata::None
                        })
                    },
                    _ => {}
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexing_operators() {
        let test = "+\0";
        let mut lex = Lexer::new(test);
        if let Some(token1) = lex.next() {
            assert_eq!(token1.token_type, TokenType::Plus);
            assert_eq!(token1.position.line, 1);
            assert_eq!(token1.position.column, 1);
        }
        if let Some(token2) = lex.next() {
            assert_eq!(token2.token_type, TokenType::Eof);
            assert_eq!(token2.position.line, 1);
            assert_eq!(token2.position.column, 2);
        }
    }

    #[test]
    fn lexing_strings() {
        let test = "\"Hello, World!\"\0";
        let mut lex = Lexer::new(test);
        if let Some(string_token) = lex.next() {
            assert_eq!(string_token.token_type, TokenType::String);
            assert_eq!(&test[string_token.lexeme.0..string_token.lexeme.1], "\"Hello, World!\"");
            assert_eq!(string_token.position.line, 1);
            assert_eq!(string_token.position.column, 1);
        }
        if let Some(eof_token) = lex.next() {
            assert_eq!(eof_token.token_type, TokenType::Eof);
            assert_eq!(eof_token.position.line, 1);
            assert_eq!(eof_token.position.column, 16);
        }
    }

    #[test]
    fn lexing_numbers() {
        let test = "12_00_00_000\0";
        let mut lex = Lexer::new(test);
        if let Some(number_token) = lex.next() {
            assert_eq!(number_token.token_type, TokenType::Number);
            assert_eq!(&test[number_token.lexeme.0..number_token.lexeme.1], "12_00_00_000");
            assert_eq!(number_token.position.line, 1);
            assert_eq!(number_token.position.column, 0);
        }
        if let Some(eof_token) = lex.next() {
            assert_eq!(eof_token.token_type, TokenType::Eof);
            assert_eq!(eof_token.position.line, 1);
            assert_eq!(eof_token.position.column, 13);
        }
    }
}
