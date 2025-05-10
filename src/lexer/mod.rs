use crate::shared::{
    errors::LexerError, positions::Position, tokens::{
        Token,
        TokenType
    }
};

#[allow(dead_code)]
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
            current_line: 0,
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
            return Ok(())
        }
        Err(LexerError::CalledNextAfterExhaustion)
    }

    fn generate_operator(&mut self, lexeme_start: usize, tt: TokenType) -> Option<Token> {
        self.advance().ok()?;
        let lexeme_ending = self.current_index;
        Some(Token {
            token_type: tt,
            position: Position::new(self.current_line, self.current_column),
            lexeme: (lexeme_start, lexeme_ending)
        })
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

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
                    '+' => return self.generate_operator(lexeme_start, TokenType::Plus),
                    '-' => return self.generate_operator(lexeme_start, TokenType::Minus),
                    '/' => return self.generate_operator(lexeme_start, TokenType::Slash),
                    '*' => return self.generate_operator(lexeme_start, TokenType::Star),
                    'a' ..= 'z' | 'A' ..= 'Z' | '_' => {
                        todo!("Create keyword here.");
                    }
                    '"' => {
                        todo!("Create string here.");
                    }
                    '0' ..= '9' => {
                        todo!("Create number here.");
                    }
                    '\0' => {}
                    _ => {}
                }
            }
        }
        None
    }
}

