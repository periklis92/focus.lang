use crate::{compiler_error::CompilerError, lexer::Lexer};

pub struct Compiler<'a> {
    lexer: Lexer<'a>,
    depth: usize,
}

impl<'a> Compiler<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
            depth: 0,
        }
    }

    pub fn parse(&mut self) -> Result<(), CompilerError> {
        todo!()
    }
}
