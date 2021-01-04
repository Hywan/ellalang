use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    NumberLit(f64),
    BoolLit(bool),
    StringLit(String),
    /// An identifier (e.g. `foo`).
    Identifier(String),
    /// A function call (e.g. `foo(1, bar, baz())`).
    FnCall {
        ident: String,
        args: Vec<Expr>,
    },
    /// A binary expression (e.g. `1+1`).
    Binary {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    Unary {
        op: Token,
        arg: Box<Expr>,
    },
    Error,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration {
        ident: String,
        initializer: Expr,
    },
    FnDeclaration {
        ident: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
    ExprStmt(Expr),
    Error,
}
