use std::{collections::HashMap, fmt::Display, ops::Range, sync::OnceLock};

macro_rules! is_reserved {
    (reserved) => {
        true
    };
    () => {
        false
    };
}

macro_rules! tuple_reserved {
    ($ident:ident $str:literal reserved) => {
        ($str.to_string(), TokenType::$ident)
    };
    ($ident:ident $str:literal) => {};
}

macro_rules! token_types {
    ($($ident:ident $str:literal $(reserved)?,)*) => {
        #[derive(Debug, Clone)]
        pub struct Token {
            pub position: usize,
            pub line: usize,
            pub column: usize,
            pub token_type: TokenType,
            pub span: Range<usize>,
        }

        #[derive(Debug, Clone, PartialEq)]
        pub enum TokenType {
            $($ident,)*
        }

        impl TokenType {
            pub fn is_reserved(&self) -> bool {
                match &self {
                    $(TokenType::$ident => is_reserved!(reserved),)*
                }
            }

            pub fn as_str(&self) -> &str {
                match &self {
                    $(TokenType::$ident => $str,)*
                }
            }
        }

        impl Display for TokenType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        static RESERVED: OnceLock<HashMap<String, TokenType>> = OnceLock::new();

        pub fn get_reserved(str: &str) -> Option<TokenType> {
            let res = RESERVED.get_or_init(|| {
                HashMap::from(
                    [$(tuple_reserved!($ident $str reserved),)*]
                )
            });

            res.get(str).cloned()
        }
    };
}

token_types!(
    Unit "()",
    Empty "<empty>",
    Colon ":",
    Comma ",",
    SingleQuote "'",
    DoubleQuote "\"",
    Dot ".",
    Dots "..",
    Spread "...",
    Plus "+",
    Minus "-",
    Div "/",
    IDiv "//",
    Mul "*",
    Mod "%",
    BinAnd "&",
    BinOr "|",
    BinXor "^",
    BinNot "~",
    Lsh "<<",
    Rsh ">>",
    Assign "=",
    Equal "==",
    Less "<",
    LessEqual "<=",
    Greater ">",
    GreaterEqual ">=",
    NotEqual "!=",
    ThinArrow "->",
    Pipe "|>",
    LBracket "[",
    RBracket "]",
    LParen "(",
    RParen ")",
    LCurly "{",
    RCurly "}",
    Ident "<ident>",
    Number "<number>",
    NewLine "<newline>",
    Eos "<end>",
    Unknown "<unknown>",
    True "true" reserved,
    False "false" reserved,
    Let "let" reserved,
    Function "fn" reserved,
    And "and" reserved,
    Not "not" reserved,
    Or "or" reserved,
    Is "is" reserved,
    Match "match" reserved,
    If "if" reserved,
    Then "then" reserved,
    Else "else" reserved,
    From "from" reserved,
    Import "import" reserved,
    As "as" reserved,
);

impl TokenType {
    pub fn is_primary(&self) -> bool {
        matches!(
            self,
            TokenType::Ident
                | TokenType::LParen
                | TokenType::Number
                | TokenType::Not
                | TokenType::Minus
                | TokenType::True
                | TokenType::False
                | TokenType::Unit
                | TokenType::DoubleQuote
                | TokenType::LCurly
                | TokenType::LBracket
                | TokenType::Function
                | TokenType::If
        )
    }
}
