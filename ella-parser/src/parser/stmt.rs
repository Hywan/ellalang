use crate::ast::DeclarationMeta;

use super::*;

impl<'a> Parser<'a> {
    /// Parses a declaration (or statement).
    pub fn parse_declaration(&mut self) -> Stmt {
        match self.current_token {
            Token::Let => self.parse_let_declaration(),
            Token::Fn => self.parse_fn_declaration(),
            _ => self.parse_stmt(),
        }
    }

    /// Parses a statement.
    pub fn parse_stmt(&mut self) -> Stmt {
        match self.current_token {
            Token::Return => self.parse_return_stmt(),
            Token::OpenBrace => self.parse_block_stmt(),
            _ => {
                // expression statement
                let expr = self.parse_expr();
                let stmt = Stmt::ExprStmt(expr);
                self.expect(Token::Semi);
                stmt
            }
        }
    }

    pub fn parse_block_stmt(&mut self) -> Stmt {
        self.expect(Token::OpenBrace);

        let mut body = Vec::new();
        if !self.eat(Token::CloseBrace) {
            loop {
                body.push(self.parse_declaration());

                if self.eat(Token::CloseBrace) {
                    break;
                } else if self.current_token == Token::Eof {
                    self.unexpected();
                    break;
                }
            }
        }

        Stmt::Block(body)
    }

    fn parse_let_declaration(&mut self) -> Stmt {
        self.expect(Token::Let);
        let ident = if let Token::Identifier(ref ident) = self.current_token {
            let ident = ident.clone();
            self.next();
            ident
        } else {
            self.unexpected();
            return Stmt::Error;
        };
        self.expect(Token::Equals);
        let initializer = self.parse_expr();
        self.expect(Token::Semi);
        Stmt::LetDeclaration {
            meta: DeclarationMeta::new(),
            ident,
            initializer,
        }
    }

    fn parse_fn_declaration(&mut self) -> Stmt {
        self.expect(Token::Fn);
        let ident = if let Token::Identifier(ref ident) = self.current_token {
            let ident = ident.clone();
            self.next();
            ident
        } else {
            self.unexpected();
            return Stmt::Error;
        };
        self.expect(Token::OpenParen);
        let mut params = Vec::new();
        if !self.eat(Token::CloseParen) {
            loop {
                params.push(if let Token::Identifier(ref ident) = self.current_token {
                    let ident = ident.clone();
                    self.next();
                    ident
                } else {
                    self.unexpected();
                    return Stmt::Error;
                });

                if self.eat(Token::CloseParen) {
                    break;
                } else if !self.eat(Token::Comma) {
                    self.unexpected();
                    break;
                }
            }
        }

        self.expect(Token::OpenBrace);

        let mut body = Vec::new();
        if !self.eat(Token::CloseBrace) {
            loop {
                body.push(self.parse_declaration());

                if self.eat(Token::CloseBrace) {
                    break;
                }
            }
        }

        Stmt::FnDeclaration {
            meta: DeclarationMeta::new(),
            body,
            ident,
            params,
        }
    }

    fn parse_return_stmt(&mut self) -> Stmt {
        self.expect(Token::Return);
        let expr = self.parse_expr();
        self.expect(Token::Semi);
        Stmt::ReturnStmt(expr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;

    fn stmt(source: &str) -> Stmt {
        let source = source.into();
        let ast = Parser::new(&source).parse_declaration();
        assert!(source.has_no_errors());
        ast
    }

    #[test]
    fn test_block_stmt() {
        assert_debug_snapshot!("block-stmt", stmt("{ 1; }"));
        assert_debug_snapshot!("block-stmt-multiple", stmt("{ 1; 2; }"));
        assert_debug_snapshot!("block-stmt-nested", stmt("{ 1; 2; { 3; } }"));
    }

    #[test]
    fn test_let_declaration() {
        assert_debug_snapshot!("let-declaration", stmt("let x = 2;"));
        assert_debug_snapshot!("let-declaration-with-expr", stmt("let x = 1 + 2;"));
    }

    #[test]
    fn test_fn_declaration() {
        assert_debug_snapshot!("fn-declaration", stmt("fn foo() {}"));
        assert_debug_snapshot!("fn-declaration-with-params", stmt("fn foo(a, b, c) {}"));
        assert_debug_snapshot!(
            "fn-declaration-with-params-and-body",
            stmt("fn foo(a, b, c) { a + b + c; }")
        );
    }

    #[test]
    fn test_return_stmt() {
        assert_debug_snapshot!("return-stmt", stmt("return 1;"));
        assert_debug_snapshot!("return-stmt-with-expr", stmt("return 1 + 2;"));
    }
}
