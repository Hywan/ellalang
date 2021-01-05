//! Lowers AST into a `Chunk` (bytecode).

use crate::{
    chunk::{Chunk, OpCode},
    value::{
        object::{Obj, ObjKind},
        Value,
    },
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

    /// Consumes `self` and returns the generated [`Chunk`].
    #[must_use]
    pub fn into_inner_chunk(self) -> Chunk {
        self.chunk
    }

    /// Returns the chunk for the top-level function.
    /// Do not use [`Visitor::visit_stmt`] to codegen a function as it will create a separate [`Chunk`].
    /// To get the generated [`Chunk`], call [`Codegen::into_inner_chunk`].
    pub fn codegen_function(&mut self, func: &mut Stmt) {
        match func {
            Stmt::FnDeclaration { body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            }
            _ => panic!("func is not a Stmt::FnDeclaration"),
        }
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
        // Do not use default walking logic.

        match stmt {
            Stmt::LetDeclaration { ident, initializer } => todo!(),
            Stmt::FnDeclaration {
                ident,
                params,
                body: _, // Body is codegen in a new `Codegen` instance.
            } => {
                let ident = ident.clone();
                let arity = params.len() as u32;

                // Create a new `Codegen` instance, codegen the function, and add the chunk to the `ObjKind::Fn`.
                let func_chunk = {
                    let mut cg = Codegen::new();
                    cg.codegen_function(stmt);
                    cg.into_inner_chunk()
                };

                let func = Rc::new(Obj {
                    kind: ObjKind::Fn {
                        ident,
                        arity,
                        chunk: func_chunk,
                    },
                });
                let constant = self.chunk.add_constant(Value::Object(func));
                self.chunk.write_chunk(OpCode::Ldc, 0);
                self.chunk.write_chunk(constant, 0);
            }
            Stmt::Block(_) => todo!(),
            Stmt::ExprStmt(expr) => {
                self.visit_expr(expr);
                self.chunk.write_chunk(OpCode::Pop, 0);
            },
            Stmt::ReturnStmt(_) => todo!(),
            Stmt::Error => {}
        }
    }
}
