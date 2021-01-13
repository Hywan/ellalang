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
            Token::If => self.parse_if_else_stmt(),
            Token::While => self.parse_while_stmt(),
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

    pub fn parse_if_else_stmt(&mut self) -> Stmt {
        self.expect(Token::If);

        let condition = self.parse_expr();
        let mut if_block = Vec::new();
        let mut else_block = None;

        self.expect(Token::OpenBrace);
        if !self.eat(Token::CloseBrace) {
            loop {
                if_block.push(self.parse_declaration());

                if self.eat(Token::CloseBrace) {
                    break;
                } else if self.current_token == Token::Eof {
                    self.unexpected();
                    break;
                }
            }
        }

        if self.eat(Token::Else) {
            else_block = Some(Vec::new());
            self.expect(Token::OpenBrace);
            if !self.eat(Token::CloseBrace) {
                loop {
                    else_block.as_mut().unwrap().push(self.parse_declaration());

                    if self.eat(Token::CloseBrace) {
                        break;
                    } else if self.current_token == Token::Eof {
                        self.unexpected();
                        break;
                    }
                }
            }
        }

        Stmt::IfElseStmt {
            condition,
            if_block,
            else_block,
        }
    }

    pub fn parse_while_stmt(&mut self) -> Stmt {
        self.expect(Token::While);
        let condition = self.parse_expr();
        let mut body = Vec::new();

        self.expect(Token::OpenBrace);
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

        Stmt::WhileStmt { condition, body }
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
        Stmt::LetDeclaration { ident, initializer }
    }

    fn parse_fn_declaration(&mut self) -> Stmt {
        self.expect(Token::Fn);
        let ident = if let Token::Identifier(ref ident) = self.current_token {
            let ident = ident.clone();
            self.next();
            ident
        } else {
            self.next();
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
    fn test_if_else_stmt() {
        assert_debug_snapshot!(
            "if-stmt",
            stmt(
                r#"
                if condition {
                    if_block();
                }"#
            )
        );
        assert_debug_snapshot!(
            "if-else-stmt",
            stmt(
                r#"
                if condition {
                    if_block();
                } else {
                    else_block();
                }"#
            )
        );
        assert_debug_snapshot!("if-else-stmt-empty", stmt(r#"if condition {} else {}"#));
    }

    #[test]
    fn test_while_stmt() {
        assert_debug_snapshot!("while-stmt", stmt(r#"while true { while_block(); }"#));
        assert_debug_snapshot!("while-stmt-empty", stmt(r#"while true {}"#));
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
