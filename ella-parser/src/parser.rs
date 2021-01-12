use crate::ast::{DeclarationMeta, Expr, Stmt};
use crate::lexer::Token;
use ella_source::{Source, SyntaxError};
use logos::{Lexer, Logos};
use std::mem;

mod expr;
mod stmt;
pub use expr::*;
pub use stmt::*;

pub struct Parser<'a> {
    /// Cached token for peeking.
    current_token: Token,
    lexer: Lexer<'a, Token>,
    /// Source code
    source: &'a Source<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a Source<'a>) -> Self {
        let mut lexer = Token::lexer(source.content);
        Self {
            current_token: lexer.next().unwrap(),
            lexer,
            source,
        }
    }
}

impl<'a> Parser<'a> {
    /// Returns an anonymous top level function.
    pub fn parse_program(&mut self) -> Stmt {
        let mut body = Vec::new();
        loop {
            body.push(self.parse_declaration());
            if self.current_token == Token::Eof {
                break;
            }
        }

        Stmt::FnDeclaration {
            meta: DeclarationMeta::new(),
            body,
            ident: "<global>".to_string(),
            params: Vec::new(),
        }
    }

    /// Returns an anonymous top level function.
    /// If the last statement is an [`Stmt::ExprStmt`], it will create a function call to `println()`.
    pub fn parse_repl_input(&mut self) -> Stmt {
        let mut body = Vec::new();
        loop {
            body.push(self.parse_declaration());
            if matches!(self.current_token, Token::Eof | Token::Error) {
                break;
            }
        }

        if let Some(Stmt::ExprStmt(expr)) = body.last_mut() {
            *expr = Expr::FnCall {
                args: vec![expr.clone()],
                ident: "println".to_string(),
            }
        }

        Stmt::FnDeclaration {
            meta: DeclarationMeta::new(),
            body,
            ident: "<global>".to_string(),
            params: Vec::new(),
        }
    }
}

/// Parse utilities
impl<'a> Parser<'a> {
    fn next(&mut self) -> Token {
        let token = self.lexer.next().unwrap_or(Token::Eof);
        self.current_token = token.clone();
        token
    }

    /// Predicate that tests whether the next token has the same discriminant and eats the next token if yes as a side effect.
    #[must_use = "to unconditionally eat a token, use Self::next"]
    fn eat(&mut self, tok: Token) -> bool {
        if mem::discriminant(&self.current_token) == mem::discriminant(&tok) {
            self.next(); // eat token
            true
        } else {
            false
        }
    }

    fn expect(&mut self, tok: Token) {
        if !self.eat(tok) {
            self.unexpected()
        }
    }

    /// Raises an unexpected token error.
    fn unexpected(&mut self) {
        self.source
            .errors
            .add_error(SyntaxError::new("Unexpected token", self.lexer.span()))
    }
}
