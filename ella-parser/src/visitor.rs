//! Visitor pattern for AST nodes.

use crate::ast::Expr;

pub trait Visitor {
    fn visit_expr(&mut self, expr: &mut Expr);
}

pub fn walk_expr(visitor: &mut impl Visitor, expr: &mut Expr) {
    match expr {
        Expr::NumberLit(_) => {}
        Expr::BoolLit(_) => {}
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
        Expr::Error => {}
    }
}
