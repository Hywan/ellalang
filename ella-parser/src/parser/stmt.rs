use super::*;

impl<'a> Parser<'a> {
    /// Parses a declaration (or statement).
    pub fn parse_declaration(&mut self) -> Stmt {
        match self.current_token {
            Token::Let => self.parse_let_declaration(),
            _ => self.parse_stmt(),
        }
    }

    /// Parses a statement.
    pub fn parse_stmt(&mut self) -> Stmt {
        match self.current_token {
            _ => {
                // expression statement
                let expr = self.parse_expr();
                let stmt = Stmt::ExprStmt(expr);
                self.expect(Token::Semi);
                stmt
            }
        }
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
}
