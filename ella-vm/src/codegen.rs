//! Lowers AST into a `Chunk` (bytecode).

use crate::{
    chunk::{Chunk, OpCode},
    value::{object::Obj, Value},
};
use ella_parser::{
    ast::{Expr, Stmt},
    lexer::Token,
    visitor::{walk_expr, walk_stmt, Visitor},
};
use std::{collections::HashMap, rc::Rc};

/// Generate bytecode from an abstract syntax tree.
pub struct Codegen {
    chunk: Chunk,
    constant_strings: HashMap<String, Rc<Obj>>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            constant_strings: HashMap::new(),
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
            Expr::StringLit(val) => {
                let obj = if let Some(obj) = self.constant_strings.get(val) {
                    // reuse same String
                    obj.clone()
                } else {
                    let obj = Rc::new(Obj::new_string(val.clone()));
                    self.constant_strings.insert(val.clone(), obj.clone());
                    obj
                };
                let constant = self.chunk.add_constant(Value::Object(obj));
                self.chunk.write_chunk(OpCode::Ldc, 0);
                self.chunk.write_chunk(constant, 0);
            }
            Expr::Identifier(_) => todo!(),
            Expr::FnCall { ident: _, args: _ } => todo!(),
            Expr::Binary { op, .. } => match op {
                Token::Plus => self.chunk.write_chunk(OpCode::Add, 0),
                Token::Minus => self.chunk.write_chunk(OpCode::Sub, 0),
                Token::Asterisk => self.chunk.write_chunk(OpCode::Mul, 0),
                Token::Slash => self.chunk.write_chunk(OpCode::Div, 0),
                Token::Equals => todo!(),
                Token::EqualsEquals => self.chunk.write_chunk(OpCode::Eq, 0),
                Token::NotEquals => {
                    self.chunk.write_chunk(OpCode::Eq, 0);
                    self.chunk.write_chunk(OpCode::Not, 0);
                }
                Token::LessThan => self.chunk.write_chunk(OpCode::Less, 0),
                Token::LessThanEquals => {
                    // a <= b equivalent to !(a > b)
                    self.chunk.write_chunk(OpCode::Greater, 0);
                    self.chunk.write_chunk(OpCode::Not, 0);
                }
                Token::GreaterThan => self.chunk.write_chunk(OpCode::Greater, 0),
                Token::GreaterThanEquals => {
                    // a >= b equivalent to !(a < b)
                    self.chunk.write_chunk(OpCode::Less, 0);
                    self.chunk.write_chunk(OpCode::Not, 0);
                }
                _ => unreachable!(),
            },
            Expr::Unary { op, .. } => match op {
                Token::LogicalNot => self.chunk.write_chunk(OpCode::Not, 0),
                Token::Minus => self.chunk.write_chunk(OpCode::Neg, 0),
                _ => unreachable!(),
            },
            Expr::Error => {}
        }
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        walk_stmt(self, stmt);
    }
}
