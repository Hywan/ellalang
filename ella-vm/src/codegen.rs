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
    visitor::{walk_expr, Visitor},
};
use ella_passes::resolve::ResolvedSymbolTable;
use std::{collections::HashMap, rc::Rc};

const DUMP_CHUNK: bool = true;

/// Generate bytecode from an abstract syntax tree.
pub struct Codegen<'a> {
    chunk: Chunk,
    constant_strings: HashMap<String, Rc<Obj>>,
    resolved_symbol_table: &'a ResolvedSymbolTable,
}

impl<'a> Codegen<'a> {
    pub fn new(name: String, resolved_symbol_table: &'a ResolvedSymbolTable) -> Self {
        Self {
            chunk: Chunk::new(name),
            constant_strings: HashMap::new(),
            resolved_symbol_table,
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

                if DUMP_CHUNK {
                    eprintln!("{}", self.chunk);
                }
            }
            _ => panic!("func is not a Stmt::FnDeclaration"),
        }
    }
}

impl<'a> Visitor for Codegen<'a> {
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
            Expr::Identifier(_) => {
                let offset = *self
                    .resolved_symbol_table
                    .get(&(expr as *const Expr))
                    .unwrap();
                self.chunk.write_chunk(OpCode::Ldloc, 0);
                self.chunk.write_chunk(offset as u8, 0);
            }
            Expr::FnCall { ident: _, args: _ } => todo!(),
            Expr::Binary { lhs, op, rhs: _ } => match op {
                Token::Plus => self.chunk.write_chunk(OpCode::Add, 0),
                Token::Minus => self.chunk.write_chunk(OpCode::Sub, 0),
                Token::Asterisk => self.chunk.write_chunk(OpCode::Mul, 0),
                Token::Slash => self.chunk.write_chunk(OpCode::Div, 0),
                Token::Equals => {
                    let offset = *self
                        .resolved_symbol_table
                        .get(&(lhs.as_ref() as *const Expr))
                        .unwrap();
                    self.chunk.write_chunk(OpCode::Stloc, 0);
                    self.chunk.write_chunk(offset as u8, 0);
                }
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
            Expr::Error => unreachable!(),
        }
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        // Do not use default walking logic.

        match stmt {
            Stmt::LetDeclaration {
                ident: _,
                initializer,
            } => {
                self.visit_expr(initializer); // Push value of expression onto top of stack.
            }
            Stmt::FnDeclaration {
                ident,
                params,
                body: _, // Body is codegen in a new `Codegen` instance.
            } => {
                let ident = ident.clone();
                let arity = params.len() as u32;

                // Create a new `Codegen` instance, codegen the function, and add the chunk to the `ObjKind::Fn`.
                let func_chunk = {
                    let mut cg = Codegen::new(ident.clone(), self.resolved_symbol_table);
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
            Stmt::Block(body) => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            }
            Stmt::ExprStmt(expr) => {
                self.visit_expr(expr);
                self.chunk.write_chunk(OpCode::Pop, 0);
            }
            Stmt::ReturnStmt(expr) => {
                self.visit_expr(expr);
                self.chunk.write_chunk(OpCode::Ret, 0);
            }
            Stmt::Error => unreachable!(),
        }
    }
}
