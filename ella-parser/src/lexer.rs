use logos::Logos;

#[derive(Debug, Logos, Clone, PartialEq)]
pub enum Token {
    // literals
    #[regex(r"[0-9.]+", |lex| lex.slice().parse())]
    NumberLit(f64),
    #[regex(r"true|false", |lex| if lex.slice() == "true" { true } else { false } )]
    BoolLit(bool),

    // identifiers
    #[regex("[a-zA-Z]+", |lex| lex.slice().to_string())]
    Identifier(String),

    // unary operators
    #[token("!")]
    LogicalNot,

    // binary operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus, // NOTE: can also be unary
    #[token("*")]
    Asterisk,
    #[token("/")]
    Slash,
    #[token("=")]
    Equals,
    #[token("==")]
    EqualsEquals,
    #[token("!=")]
    NotEquals,

    // punctuation
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token(",")]
    Comma,

    // keywords
    #[token("fn")]
    Fn,
    #[token("return")]
    Return,

    // misc
    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[error]
    Error,

    /// Only generated in parse phase when `lexer.next()` returns `None`.
    Eof,
}

impl Token {
    /// Returns the binary binding power or `None` if invalid binop token.
    /// Binding power `0` and `1` is reserved for accepting any expression.
    /// Assignment (`Token::Equals`) has the lowest precedence with `(3, 2)`.
    pub fn binop_bp(&self) -> Option<(u8, u8)> {
        match self {
            /* Additive */
            Token::Plus => Some((6, 7)),
            Token::Minus => Some((6, 7)),
            /* Multiplicative */
            Token::Asterisk => Some((8, 9)),
            Token::Slash => Some((8, 9)),
            /* Assignment */
            Token::Equals => Some((3, 2)),
            /* Equality */
            Token::EqualsEquals => Some((4, 5)),
            Token::NotEquals => Some((4, 5)),
            _ => None,
        }
    }
}
