use crate::ast::Expr;
use crate::lexer::Token;
use ella_source::{Source, SyntaxError};
use logos::{Lexer, Logos};
use std::mem;

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
    pub fn parse_program(&mut self) -> Expr {
        self.parse_expr()
    }

    /* Expressions */
    pub fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(0) // 0 to accept any expression
    }

    pub fn parse_primary_expr(&mut self) -> Expr {
        match self.current_token {
            Token::NumberLit(_) | Token::BoolLit(_) => self.parse_literal_expr(),
            Token::Identifier(_) => self.parse_identifier_expr(),
            _ => {
                self.unexpected();
                Expr::Error
            }
        }
    }

    pub fn parse_expr_bp(&mut self, min_bp: u8) -> Expr {
        let mut lhs = self.parse_primary_expr();

        loop {
            let (l_bp, r_bp) = match self.current_token.binop_bp() {
                Some(bp) => bp,
                None => break, // not a valid binop, stop parsing
            };
            if l_bp < min_bp {
                break; // less than the min_bp, stop parsing
            }

            // self.current_token is a valid binop
            let binop = self.current_token.clone();
            self.next();

            let rhs = self.parse_expr_bp(r_bp);

            lhs = Expr::Binary {
                lhs: Box::new(lhs),
                op: binop,
                rhs: Box::new(rhs),
            }
        }

        lhs
    }

    /* Expressions.Literals */
    /// Parses a literal expression.
    /// A literal can be either a number literal or a bool literal.
    pub fn parse_literal_expr(&mut self) -> Expr {
        let val = match self.current_token {
            Token::NumberLit(val) => Expr::NumberLit(val),
            Token::BoolLit(val) => Expr::BoolLit(val),
            _ => {
                self.unexpected();
                Expr::Error
            }
        };
        if val != Expr::Error {
            self.next(); // eat parsed token if not error
        }
        val
    }

    /* Expressions.Identifier */
    pub fn parse_identifier_expr(&mut self) -> Expr {
        match self.current_token.clone() {
            Token::Identifier(ident) => {
                self.next();
                Expr::Identifier(ident)
            }
            _ => {
                self.unexpected();
                Expr::Error
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    fn expr(source: &str) -> Expr {
        let source = source.into();
        Parser::new(&source).parse_expr()
    }

    #[test]
    fn test_literal() {
        assert_debug_snapshot!("bool-lit-true", expr("true"));
        assert_debug_snapshot!("bool-lit-false", expr("false"));
        assert_debug_snapshot!("int", expr("1"));
        assert_debug_snapshot!("double-2.0", expr("2.0"));
        assert_debug_snapshot!("double-2.5", expr("2.5"));
    }

    #[test]
    fn test_binary_expr() {
        assert_debug_snapshot!("binary", expr("1 + 1"));
        assert_debug_snapshot!("binary-equality", expr("1 == 2 - 1"));
        assert_debug_snapshot!("binary-associativity", expr("2 * 2 * 2")); // should be (2 * 2) * 2
        assert_debug_snapshot!("binary-associativity-2", expr("a = b = c")); // should be a = (b = c)
    }
}
