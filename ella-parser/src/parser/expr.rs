use super::*;

impl<'a> Parser<'a> {
    /* Expressions */
    /// Parses any expression.
    /// This is equivalent to calling [`Self::parse_expr_bp`] with `min_bp = 0`.
    pub fn parse_expr(&mut self) -> Expr {
        self.parse_expr_bp(0) // 0 to accept any expression
    }

    /// Parses a primary (atom) expression.
    fn parse_primary_expr(&mut self) -> Expr {
        // NOTE: prefix operators are handled here
        match self.current_token {
            Token::NumberLit(_) | Token::BoolLit(_) | Token::StringLit(_) => self.parse_literal_expr(),
            Token::Identifier(_) => self.parse_identifier_or_call_expr(),
            Token::LogicalNot => {
                self.next();
                Expr::Unary {
                    op: Token::LogicalNot,
                    arg: Box::new(self.parse_expr()),
                }
            }
            Token::Minus => {
                self.next();
                Expr::Unary {
                    op: Token::Minus,
                    arg: Box::new(self.parse_expr()),
                }
            }
            _ => {
                self.unexpected();
                Expr::Error
            }
        }
    }

    /// Parses an expression with the specified `min_bp`.
    /// To parse any expression use, [`Self::parse_expr`].
    fn parse_expr_bp(&mut self, min_bp: u8) -> Expr {
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
    fn parse_literal_expr(&mut self) -> Expr {
        let val = match self.current_token {
            Token::NumberLit(val) => Expr::NumberLit(val),
            Token::BoolLit(val) => Expr::BoolLit(val),
            Token::StringLit(ref val) => Expr::StringLit(val.clone()),
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
    /// Parses an identifier or a call expression.
    fn parse_identifier_or_call_expr(&mut self) -> Expr {
        let ident = match self.current_token.clone() {
            Token::Identifier(ident) => {
                self.next();
                ident
            }
            _ => {
                self.unexpected();
                return Expr::Error;
            }
        };

        if self.eat(Token::OpenParen) {
            // parse call expression
            let mut args = Vec::new();

            if !self.eat(Token::CloseParen) {
                loop {
                    args.push(self.parse_expr());

                    if self.eat(Token::CloseParen) {
                        break;
                    } else if !self.eat(Token::Comma) {
                        self.unexpected();
                        break;
                    }
                }
            }

            Expr::FnCall { ident, args }
        } else {
            // parse identifier expression
            Expr::Identifier(ident)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    fn expr(source: &str) -> Expr {
        let source = source.into();
        let ast = Parser::new(&source).parse_expr();
        assert!(source.has_no_errors());
        ast
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

    #[test]
    fn test_identifier() {
        assert_debug_snapshot!("identifier", expr("foo"));
    }

    #[test]
    fn test_fn_call() {
        assert_debug_snapshot!("fn-call", expr("foo()"));
        assert_debug_snapshot!("fn-call-with-args", expr("foo(1, bar)"));
        assert_debug_snapshot!("fn-call-with-nested-args", expr("foo(1, bar, baz())"));
    }
}
