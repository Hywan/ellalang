//! Lowers AST into a `Chunk` (bytecode).

use ella_parser::{
    ast::{Expr, Stmt},
    lexer::Token,
    visitor::{walk_expr, Visitor},
};
use ella_passes::resolve::{ResolvedSymbolTable, Symbol, SymbolTable};
use ella_value::chunk::{Chunk, OpCode};
use ella_value::object::{Function, Obj, ObjKind};
use ella_value::{BuiltinVars, Value};
use std::cell::RefCell;
use std::{collections::HashMap, rc::Rc};

const DUMP_CHUNK: bool = true;

/// Generate bytecode from an abstract syntax tree.
pub struct Codegen<'a> {
    chunk: Chunk,
    constant_strings: HashMap<String, Rc<Obj>>,
    symbol_table: &'a SymbolTable,
    resolved_symbol_table: &'a ResolvedSymbolTable,
    /// Every time a new scope is created, a new value is pushed onto the stack.
    /// This is to keep track of how many `pop` instructions to emit when exiting the scope.
    scope_stack: Vec<Vec<Rc<RefCell<Symbol>>>>,
}

impl<'a> Codegen<'a> {
    pub fn new(
        name: String,
        symbol_table: &'a SymbolTable,
        resolved_symbol_table: &'a ResolvedSymbolTable,
    ) -> Self {
        Self {
            chunk: Chunk::new(name),
            constant_strings: HashMap::new(),
            symbol_table,
            resolved_symbol_table,
            scope_stack: vec![Vec::new()],
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
    /// # Params
    /// * `func` - The function to codegen for.
    pub fn codegen_function(&mut self, func: &'a Stmt) {
        match func {
            Stmt::FnDeclaration { body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            }
            _ => panic!("func is not a Stmt::FnDeclaration"),
        }

        if DUMP_CHUNK {
            eprintln!("{}", self.chunk);
        }
    }

    pub fn codegen_builtin_vars(&mut self, builtin_vars: &BuiltinVars) {
        for (_ident, value) in &builtin_vars.values {
            let constant = self.chunk.add_constant(value.clone());
            self.chunk.write_chunk(OpCode::Ldc, 0);
            self.chunk.write_chunk(constant, 0);
        }
    }

    fn enter_scope(&mut self) {
        self.scope_stack.push(Vec::new());
    }

    fn add_symbol(&mut self, stmt: &Stmt) {
        let stmt = &(stmt as *const Stmt);
        let symbol = self.symbol_table.get(stmt).unwrap();
        self.scope_stack
            .last_mut()
            .unwrap()
            .push(Rc::clone(&symbol));
    }

    fn exit_scope(&mut self) {
        let scope = self.scope_stack.pop().unwrap();
        for symbol in scope {
            match symbol.borrow().is_captured {
                true => self.chunk.write_chunk(OpCode::CloseUpVal, 0),
                false => self.chunk.write_chunk(OpCode::Pop, 0),
            }
        }
    }
}

impl<'a> Visitor<'a> for Codegen<'a> {
    fn visit_expr(&mut self, expr: &'a Expr) {
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
                let resolved_symbol = *self
                    .resolved_symbol_table
                    .get(&(expr as *const Expr))
                    .unwrap();
                match resolved_symbol.is_upvalue {
                    true => {
                        self.chunk.write_chunk(OpCode::LdUpVal, 0);
                        self.chunk.write_chunk(resolved_symbol.offset as u8, 0);
                    }
                    false => {
                        self.chunk.write_chunk(OpCode::LdLoc, 0);
                        self.chunk.write_chunk(resolved_symbol.offset as u8, 0);
                    }
                }
            }
            Expr::FnCall { ident: _, args } => {
                let arity = args.len() as u8;
                let resolved_symbol = *self
                    .resolved_symbol_table
                    .get(&(expr as *const Expr))
                    .unwrap();
                match resolved_symbol.is_upvalue {
                    true => {
                        self.chunk.write_chunk(OpCode::LdUpVal, 0);
                        self.chunk.write_chunk(resolved_symbol.offset as u8, 0);
                    }
                    false => {
                        self.chunk.write_chunk(OpCode::LdLoc, 0);
                        self.chunk.write_chunk(resolved_symbol.offset as u8, 0);
                    }
                }
                self.chunk.write_chunk(OpCode::Calli, 0);
                self.chunk.write_chunk(arity, 0);
            }
            Expr::Binary { lhs, op, rhs: _ } => match op {
                Token::Plus => self.chunk.write_chunk(OpCode::Add, 0),
                Token::Minus => self.chunk.write_chunk(OpCode::Sub, 0),
                Token::Asterisk => self.chunk.write_chunk(OpCode::Mul, 0),
                Token::Slash => self.chunk.write_chunk(OpCode::Div, 0),
                Token::Equals => {
                    let resolved_symbol = *self
                        .resolved_symbol_table
                        .get(&(lhs.as_ref() as *const Expr))
                        .unwrap();
                    match resolved_symbol.is_upvalue {
                        true => {
                            self.chunk.write_chunk(OpCode::StUpVal, 0);
                            self.chunk.write_chunk(resolved_symbol.offset as u8, 0);
                        }
                        false => {
                            self.chunk.write_chunk(OpCode::StLoc, 0);
                            self.chunk.write_chunk(resolved_symbol.offset as u8, 0);
                        }
                    }
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

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        // Do not use default walking logic.

        match stmt {
            Stmt::LetDeclaration {
                ident: _,
                initializer,
            } => {
                self.visit_expr(initializer); // Push value of expression onto top of stack.
                self.add_symbol(stmt);
            }
            Stmt::FnDeclaration {
                ident,
                params,
                body: _, // Body is codegen in a new `Codegen` instance.
            } => {
                // NOTE: we don't need to create a new scope here because the VM automatically cleans up the created local variables.
                let ident = ident.clone();
                let arity = params.len() as u32;

                // Create a new `Codegen` instance, codegen the function, and add the chunk to the `ObjKind::Fn`.
                let fn_chunk = {
                    let mut cg =
                        Codegen::new(ident.clone(), self.symbol_table, self.resolved_symbol_table);
                    cg.codegen_function(stmt);
                    cg.chunk
                };

                let symbol = self.symbol_table.get(&(stmt as *const Stmt)).unwrap();

                let func = Rc::new(Obj {
                    kind: ObjKind::Fn(Function {
                        ident,
                        arity,
                        chunk: fn_chunk,
                        upvalues_count: symbol.borrow().upvalues.len(),
                    }),
                });
                let constant = self.chunk.add_constant(Value::Object(func));
                self.chunk.write_chunk(OpCode::Closure, 0);
                self.chunk.write_chunk(constant, 0);

                for symbol in &symbol.borrow().upvalues {
                    self.chunk.write_chunk(symbol.is_local as u8, 0);
                    self.chunk.write_chunk(symbol.index as u8, 0);
                }
            }
            Stmt::Block(body) => {
                self.enter_scope();
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.exit_scope();
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
