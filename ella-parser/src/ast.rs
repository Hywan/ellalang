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

/// Metadata related to variable declarations.
#[derive(Debug, Clone, PartialEq)]
pub struct DeclarationMeta {
    /// *Default:* `None`
    /// Modified in variable resolution pass.
    pub is_captured_in_closure: Option<bool>,
}

impl DeclarationMeta {
    pub fn new() -> Self {
        Self {
            is_captured_in_closure: Some(false),
        }
    }
}

impl Default for DeclarationMeta {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetDeclaration {
        meta: DeclarationMeta,
        ident: String,
        initializer: Expr,
    },
    FnDeclaration {
        meta: DeclarationMeta,
        ident: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Block(Vec<Stmt>),
    ExprStmt(Expr),
    ReturnStmt(Expr),
    Error,
}
