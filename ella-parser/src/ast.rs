//! AST (abstract syntax tree) data structure.

use crate::lexer::Token;

/// Represents an expression node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Number literal (represented using floating point `f64`).
    NumberLit(f64),
    /// Boolean literal.
    BoolLit(bool),
    /// String literal.
    StringLit(String),
    /// An identifier (e.g. `foo`).
    Identifier(String),
    /// A function call (e.g. `foo(1, bar, baz())`).
    FnCall {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    /// A binary expression (e.g. `1+1`).
    Binary {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    /// An unary expression (e.g. `-1`).
    Unary {
        op: Token,
        arg: Box<Expr>,
    },
    /// Error token. Used for error recovery.
    Error,
}

/// Represents a statement node in the AST.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Variable declaration.
    LetDeclaration {
        ident: String,
        initializer: Expr,
    },
    /// Function declaration.
    FnDeclaration {
        ident: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    /// Block statement.
    Block(Vec<Stmt>),
    /// If/else statement.
    IfElseStmt {
        condition: Expr,
        if_block: Vec<Stmt>,
        /// If `else` clause is not present, this field should be `None`.
        else_block: Option<Vec<Stmt>>,
    },
    /// Expression statement (expression with side effect).
    ExprStmt(Expr),
    /// Return statement.
    ReturnStmt(Expr),
    /// Error token. Used for error recovery/
    Error,
}
