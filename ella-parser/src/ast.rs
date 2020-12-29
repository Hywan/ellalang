use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    NumberLit(f64),
    BoolLit(bool),
    /// An identifier (e.g. `foo`).
    Identifier(String),
    /// A binary expression (e.g. `1+1`).
    Binary {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    Error,
}
