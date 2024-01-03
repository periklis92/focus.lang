use std::ops::Range;

use crate::token::{get_reserved, Token, TokenType};

#[derive(Clone)]
pub struct Lexer<'a> {
    source: &'a str,
    position: usize,
    last_space: usize,
    indentation: usize,
    is_new_line: bool,
    last_token: Token,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            position: 0,
            last_space: 0,
            indentation: 0,
            is_new_line: true,
            last_token: Token {
                position: 0,
                line: 1,
                column: 0,
                token_type: TokenType::Eos,
                span: 0..0,
            },
            line: 0,
            column: 0,
        }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn last_token(&self) -> Token {
        self.last_token.clone()
    }

    pub fn whitespace(&self) -> usize {
        self.last_space
    }

    pub fn indentation(&self) -> usize {
        self.indentation
    }

    pub fn slice(&self, range: Range<usize>) -> &str {
        &self.source[range]
    }

    pub fn source(&self) -> &str {
        self.source
    }

    pub fn peek(&self) -> TokenType {
        self.peek_nth(0)
    }

    pub fn peek_empty(&self) -> TokenType {
        self.clone().next_empty().token_type
    }

    pub fn peek_indented(&self) -> Option<TokenType> {
        let mut l = self.clone();
        l.next_indented().map(|t| t.token_type)
    }

    pub fn peek_indentation(&self) -> usize {
        let mut l = self.clone();
        l.skip_new_lines();
        if l.next().token_type == TokenType::Eos {
            return 0;
        } else {
            l.indentation
        }
    }

    pub fn peek_nth(&self, n: usize) -> TokenType {
        let mut l = self.clone();
        for _ in 0..=n {
            l.next();
        }
        l.last_token.token_type
    }

    pub fn peek_continued(&self, indentation: usize) -> Option<Token> {
        let mut cloned = self.clone();
        if cloned.peek() == TokenType::NewLine {
            if cloned.peek_indentation() == indentation {
                cloned.skip_new_lines();
                Some(cloned.next())
            } else {
                None
            }
        } else {
            Some(cloned.next())
        }
    }

    pub fn next_checked(&mut self, token_type: TokenType) -> Option<Token> {
        if self.peek() == token_type {
            Some(self.next())
        } else {
            None
        }
    }

    pub fn next_checked_indented(&mut self, token_type: TokenType) -> Option<Token> {
        if self.peek_indented().is_some_and(|t| t == token_type) {
            self.next_indented()
        } else {
            None
        }
    }

    pub fn next_checked_empty(&mut self, token_type: TokenType) -> Option<Token> {
        if self.peek_empty() == token_type {
            Some(self.next_empty())
        } else {
            None
        }
    }

    pub fn next_checked_continued(
        &mut self,
        token_type: TokenType,
        indentation: usize,
    ) -> Option<Token> {
        if self
            .peek_continued(indentation)
            .is_some_and(|t| t.token_type == token_type)
        {
            self.next_continued(indentation)
        } else {
            None
        }
    }

    pub fn next(&mut self) -> Token {
        self.next_internal(true)
    }

    pub fn next_empty(&mut self) -> Token {
        self.next_internal(false)
    }

    pub fn next_indented(&mut self) -> Option<Token> {
        if self.next_checked(TokenType::NewLine).is_some() {
            if self.peek_indentation() > self.indentation {
                Some(self.next())
            } else {
                None
            }
        } else {
            Some(self.next())
        }
    }

    pub fn next_continued(&mut self, indentation: usize) -> Option<Token> {
        if self.peek() == TokenType::NewLine {
            if self.peek_indentation() == indentation {
                self.skip_new_lines();
                Some(self.next())
            } else {
                None
            }
        } else {
            Some(self.next())
        }
    }

    fn next_internal(&mut self, skip_empty: bool) -> Token {
        let whitespace = self.count_whitespace();
        self.position += whitespace;
        self.last_space = whitespace;
        self.column += whitespace;

        if self.is_new_line {
            self.indentation = whitespace;
            self.is_new_line = false;
        }

        if whitespace > 0 && !skip_empty {
            return Token {
                position: self.position - whitespace,
                line: self.line,
                column: self.column - whitespace,
                token_type: TokenType::Empty,
                span: self.position - whitespace..self.position,
            };
        }

        let start_position = self.position;
        let start_line = self.line;
        let start_col = self.column;

        let Some(ch) = self.next_char() else {
            let token = Token {
                position: start_position,
                line: self.line,
                column: self.column,
                token_type: TokenType::Eos,
                span: start_position..self.position,
            };
            self.last_token = token.clone();
            return token;
        };

        let token = match ch {
            '\n' => {
                self.is_new_line = true;
                self.line += 1;
                self.column = 0;
                TokenType::NewLine
            }
            '#' => TokenType::Hash,
            ':' => TokenType::Colon,
            ',' => TokenType::Comma,
            '!' if self.next_char_checked('=') => TokenType::NotEqual,
            '+' => TokenType::Plus,
            '=' if self.next_char_checked('=') => TokenType::Equal,
            '=' => TokenType::Assign,
            '-' if self.next_char_checked('>') => TokenType::ThinArrow,
            '-' => TokenType::Minus,
            '*' => TokenType::Mul,
            '%' => TokenType::Mod,
            '/' if self.next_char_checked('/') => TokenType::IDiv,
            '/' => TokenType::Div,
            '"' => TokenType::DoubleQuote,
            '\'' => TokenType::SingleQuote,
            '.' if self.next_char_checked('.') => {
                if self.next_char_checked('.') {
                    TokenType::Spread
                } else {
                    TokenType::Dots
                }
            }
            '.' => TokenType::Dot,
            '<' if self.next_char_checked('<') => TokenType::Lsh,
            '<' if self.next_char_checked('=') => TokenType::LessEqual,
            '<' => TokenType::Less,
            '>' if self.next_char_checked('>') => TokenType::Rsh,
            '>' if self.next_char_checked('=') => TokenType::GreaterEqual,
            '>' => TokenType::Greater,
            '{' => TokenType::LCurly,
            '}' => TokenType::RCurly,
            '[' => TokenType::LBracket,
            ']' => TokenType::RBracket,
            '(' if self.next_char_checked(')') => TokenType::Unit,
            '(' => TokenType::LParen,
            ')' => TokenType::RParen,
            '&' => TokenType::BinAnd,
            '|' if self.next_char_checked('>') => TokenType::Pipe,
            '|' => TokenType::BinAnd,
            '^' => TokenType::BinXor,
            '~' => TokenType::BinNot,
            c if c.is_numeric() => {
                let len = self
                    .source
                    .chars()
                    .skip(self.position)
                    .take_while(|c| c.is_numeric() || *c == '_' || *c == '.')
                    .count();
                self.position += len;
                TokenType::Number
            }
            c if c.is_alphabetic() || c == '_' => {
                let len = self
                    .source
                    .chars()
                    .skip(self.position)
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .count();
                self.position += len;
                let str = &self.source[start_position..self.position];
                if let Some(tok) = get_reserved(str) {
                    tok
                } else {
                    TokenType::Ident
                }
            }
            _ => TokenType::Unknown,
        };
        if !self.is_new_line {
            self.column += self.position - start_position;
        }
        let token = Token {
            position: start_position,
            line: start_line,
            column: start_col,
            token_type: token,
            span: start_position..self.position,
        };
        self.last_token = token.clone();
        token
    }

    pub fn skip_new_lines(&mut self) {
        while self.peek() == TokenType::NewLine {
            self.next();
        }
    }

    pub fn skip_line(&mut self) {
        while self.peek() != TokenType::NewLine {
            self.next();
        }
        self.next();
    }

    pub fn skip_comments_and_new_lines(&mut self) {
        loop {
            let peek = self.peek();
            if peek == TokenType::Hash {
                self.skip_line();
            } else if peek == TokenType::NewLine {
                self.skip_new_lines();
            } else {
                break;
            }
        }
    }

    fn count_whitespace(&self) -> usize {
        self.source
            .chars()
            .skip(self.position)
            .enumerate()
            .take_while(|(_, c)| c.is_whitespace() && *c != '\n')
            .count()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.source.chars().nth(self.position);
        self.position += 1;
        ch
    }

    fn next_char_checked(&mut self, ch: char) -> bool {
        let next = self.source.chars().nth(self.position);
        if next.is_some_and(|c| c == ch) {
            self.position += 1;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::{Token, TokenType};

    use super::Lexer;

    #[test]
    fn dots() {
        let mut lexer = Lexer::new(". .. ...");
        assert_eq!(lexer.next().token_type, TokenType::Dot);
        assert_eq!(lexer.next().token_type, TokenType::Dots);
        assert_eq!(lexer.next().token_type, TokenType::Spread);
    }

    #[test]
    fn whitespace() {
        let mut lexer = Lexer::new("  a   \nb");
        let i = lexer.count_whitespace();
        assert_eq!(2, i);
        lexer.position += i + 1;
        let i = lexer.count_whitespace();
        assert_eq!(3, i);
        lexer.position += i + 1;
        let i = lexer.count_whitespace();
        assert_eq!(0, i);
    }

    #[test]
    fn reserved() {
        let mut lexer = Lexer::new("let a");
        assert_eq!(lexer.next().token_type, TokenType::Let);
        assert_eq!(lexer.next().token_type, TokenType::Ident);
    }

    #[test]
    fn identifier() {
        let mut lexer = Lexer::new("this_is_a_name_2");
        let token = lexer.next();
        assert_eq!(token.token_type, TokenType::Ident);
        let slice = lexer.slice(token.span);
        assert_eq!(slice, "this_is_a_name_2");
    }

    #[test]
    fn peek() {
        let mut lexer = Lexer::new("hello.world 123 not");
        lexer.next();
        assert_eq!(lexer.peek(), TokenType::Dot);
        assert_eq!(lexer.peek_nth(3), TokenType::Not);
    }

    #[test]
    fn line_col_number() {
        let mut lexer = Lexer::new("hello brave\nnew world!");
        assert!(matches!(
            lexer.next(),
            Token {
                line: 0,
                column: 0,
                ..
            }
        ));
        assert!(matches!(
            lexer.next(),
            Token {
                line: 0,
                column: 6,
                ..
            }
        ));
        lexer.next(); //Skip new line
        assert!(matches!(
            lexer.next(),
            Token {
                line: 1,
                column: 0,
                ..
            }
        ));
        assert!(matches!(
            lexer.next(),
            Token {
                line: 1,
                column: 4,
                ..
            }
        ));
    }
}
