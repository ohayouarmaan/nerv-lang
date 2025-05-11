#[allow(dead_code)]
pub enum LexerError {
    CalledNextAfterExhaustion,
    IllegalCharacterAccess,
    IllegalNumber,
    UnexpectedEof
}
