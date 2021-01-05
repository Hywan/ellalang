//! Visitor pattern for AST nodes.

use crate::ast::{Expr, Stmt};

pub trait Visitor: Sized {
    fn visit_expr(&mut self, expr: &mut Expr) {
        walk_expr(self, expr);
    }
    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        walk_stmt(self, stmt);
    }
}

pub fn walk_expr(visitor: &mut impl Visitor, expr: &mut Expr) {
    match expr {
        Expr::NumberLit(_) => {}
        Expr::BoolLit(_) => {}
        Expr::StringLit(_) => {}
        Expr::Identifier(_) => {}
        Expr::FnCall { ident: _, args } => {
            for arg in args {
                visitor.visit_expr(arg);
            }
        }
        Expr::Binary { lhs, op: _, rhs } => {
            visitor.visit_expr(lhs);
            visitor.visit_expr(rhs);
        }
        Expr::Unary { op: _, arg } => visitor.visit_expr(arg),
        Expr::Error => {}
    }
}

pub fn walk_stmt(visitor: &mut impl Visitor, stmt: &mut Stmt) {
    /// Iteratively visit all statements in a `Vec<Stmt>`.
    macro_rules! visit_stmt_list {
        ($visitor: expr, $body: expr) => {
            for stmt in $body {
                Visitor::visit_stmt($visitor, stmt);
            }
        };
    }

    match stmt {
        Stmt::LetDeclaration {
            ident: _,
            initializer,
        } => visitor.visit_expr(initializer),
        Stmt::FnDeclaration {
            ident: _,
            params: _,
            body,
        } => visit_stmt_list!(visitor, body),
        Stmt::Block(body) => visit_stmt_list!(visitor, body),
        Stmt::ExprStmt(expr) => visitor.visit_expr(expr),
        Stmt::ReturnStmt(expr) => visitor.visit_expr(expr),
        Stmt::Error => {}
    }
}
