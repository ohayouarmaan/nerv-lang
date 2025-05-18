#[derive(Debug)]
pub enum LexerError {
    CalledNextAfterExhaustion,
    IllegalCharacterAccess,
    IllegalNumber,
    UnexpectedEof,
    IllegalKeyword
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum CompilerError {
    IllegalOutputFile,
    CanNotWrite,
}

