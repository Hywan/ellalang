//! Lowers AST into a `Chunk` (bytecode).

use ella_parser::{
    ast::Expr,
    lexer::Token,
    visitor::{walk_expr, Visitor},
};

use crate::{
    chunk::{Chunk, OpCode},
    value::Value,
};

/// Generate bytecode from an abstract syntax tree.
pub struct Codegen {
    chunk: Chunk,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
        }
    }

    /// Consumes `self` and returns the generated `chunk`.
    pub fn into_inner_chunk(self) -> Chunk {
        self.chunk
    }
}

impl Visitor for Codegen {
    fn visit_expr(&mut self, expr: &mut Expr) {
        walk_expr(self, expr);

        match expr {
            Expr::NumberLit(val) => {
                let constant = self.chunk.add_constant(Value::Number(*val));
                self.chunk.write_chunk(OpCode::Ldc, 0);
                self.chunk.write_chunk(constant, 0);
            }
            Expr::BoolLit(val) => match val {
                true => self.chunk.write_chunk(OpCode::LdTrue, 0),
                false => self.chunk.write_chunk(OpCode::LdFalse, 0),
            },
            Expr::Identifier(_) => todo!(),
            Expr::FnCall { ident: _, args: _ } => todo!(),
            Expr::Binary { op, .. } => match op {
                Token::Plus => self.chunk.write_chunk(OpCode::Add, 0),
                Token::Minus => self.chunk.write_chunk(OpCode::Sub, 0),
                Token::Asterisk => self.chunk.write_chunk(OpCode::Mul, 0),
                Token::Slash => self.chunk.write_chunk(OpCode::Div, 0),
                Token::Equals => todo!(),
                Token::EqualsEquals => todo!(),
                Token::NotEquals => todo!(),
                _ => unreachable!(),
            },
            Expr::Unary { op, .. } => match op {
                Token::LogicalNot => self.chunk.write_chunk(OpCode::Not, 0),
                _ => unreachable!(),
            },
            Expr::Error => {}
        }
    }
}
