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
        Stmt::LetDeclaration { ident, initializer }
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
