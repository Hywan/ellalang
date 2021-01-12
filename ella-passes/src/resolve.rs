//! Variable resolution pass.

use std::cell::RefCell;
use std::collections::HashMap;
use std::mem;
use std::ops::Range;
use std::rc::Rc;

use ella_parser::ast::{Expr, Stmt};
use ella_parser::visitor::{walk_expr, Visitor};
use ella_source::{Source, SyntaxError};
use ella_value::BuiltinVars;

/// Represents a symbol (created using `let` or `fn` declaration statement).
#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    ident: String,
    scope_depth: u32,
    pub is_captured: bool,
    pub upvalues: Vec<ResolvedUpValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedUpValue {
    pub is_local: bool,
    pub index: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedSymbol {
    /// The offset relative to the current function's offset (`current_func_offset`).
    pub offset: i32,
    pub is_upvalue: bool,
}

pub type SymbolTable = HashMap<*const Stmt, Rc<RefCell<Symbol>>>;
pub type ResolvedSymbolTable = HashMap<*const Expr, ResolvedSymbol>;

/// Variable resolution pass.
pub struct Resolver<'a> {
    /// A [`HashMap`] mapping all declaration [`Stmt`]s to [`Symbol`]s.
    symbol_table: SymbolTable,
    /// A [`HashMap`] mapping all [`Expr::Identifier`]s to [`ResolvedSymbol`]s.
    resolved_symbol_table: ResolvedSymbolTable,
    /// A [`Vec`] of symbols that are currently in (lexical) scope.
    accessible_symbols: Vec<Rc<RefCell<Symbol>>>,
    /// The current scope depth. `0` is global scope.
    current_scope_depth: u32,
    /// Every time a new function scope is created, `current_func_offset` should be set to `self.resolved_symbols.len()`.
    /// When exiting a function scope, the value should be reverted to previous value.
    current_func_offset: i32,
    current_upvalues: Vec<ResolvedUpValue>,
    source: &'a Source<'a>,
}

impl<'a> Resolver<'a> {
    pub fn new(source: &'a Source) -> Self {
        Self {
            symbol_table: SymbolTable::new(),
            resolved_symbol_table: ResolvedSymbolTable::new(),
            accessible_symbols: Vec::new(),
            current_scope_depth: 0,
            current_func_offset: 0,
            current_upvalues: Vec::new(),
            source,
        }
    }

    pub fn new_with_existing_accessible_symbols(
        source: &'a Source,
        resolved_symbols: Vec<Rc<RefCell<Symbol>>>,
    ) -> Self {
        Self {
            accessible_symbols: resolved_symbols,
            ..Self::new(source)
        }
    }

    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// Returns a [`HashMap`] mapping all [`Expr::Identifier`] to variable offsets.
    pub fn resolved_symbol_table(&self) -> &ResolvedSymbolTable {
        &self.resolved_symbol_table
    }

    pub fn accessible_symbols(&self) -> &Vec<Rc<RefCell<Symbol>>> {
        &self.accessible_symbols
    }

    fn enter_scope(&mut self) {
        self.current_scope_depth += 1;
    }

    fn exit_scope(&mut self) {
        self.current_scope_depth -= 1;

        // Remove all symbols in current scope.
        self.accessible_symbols = self
            .accessible_symbols
            .iter()
            .filter(|symbol| symbol.borrow().scope_depth <= self.current_scope_depth)
            .cloned()
            .collect();
    }

    /// Adds a symbol to `self.accessible_symbols`.
    fn add_symbol(&mut self, ident: String, stmt: Option<&Stmt>) {
        let symbol = Rc::new(RefCell::new(Symbol {
            ident,
            scope_depth: self.current_scope_depth,
            is_captured: false, // not captured by default
            upvalues: Vec::new(),
        }));
        self.accessible_symbols.push(Rc::clone(&symbol));
        if let Some(stmt) = stmt {
            self.symbol_table.insert(stmt as *const Stmt, symbol);
        }
    }

    /// Returns a `Some((usize, Rc<RefCell<Symbol>>))` or `None` if cannot be resolved.
    /// The `usize` is the offset of the variable.
    ///
    /// # Params
    /// * `ident` - The identifier to resolve.
    /// * `span` - The span of the expression to resolve. This is used for error reporting in case the variable could not be resolved.
    fn resolve_symbol(
        &mut self,
        ident: &str,
        span: Range<usize>,
    ) -> Option<(usize, Rc<RefCell<Symbol>>)> {
        for (i, symbol) in self.accessible_symbols.iter_mut().enumerate().rev() {
            if symbol.borrow().ident == ident {
                if symbol.borrow().scope_depth == self.current_scope_depth {
                    return Some((i - self.current_func_offset as usize, symbol.clone()));
                } else {
                    // capture outer variable
                    symbol.borrow_mut().is_captured = true;
                    self.current_upvalues.push(ResolvedUpValue {
                        is_local: true, // TODO
                        index: i as i32,
                    });
                    return Some(((self.current_upvalues.len() - 1) as usize, symbol.clone()));
                }
            }
        }
        self.source.errors.add_error(SyntaxError::new(
            format!("Cannot resolve symbol {}", ident),
            span,
        ));
        None
    }

    pub fn resolve_top_level(&mut self, func: &'a Stmt) {
        match func {
            Stmt::FnDeclaration { body, .. } => {
                for stmt in body {
                    self.visit_stmt(stmt);
                }
            }
            _ => panic!("func is not a Stmt::FnDeclaration"),
        }
    }

    pub fn resolve_builtin_vars(&mut self, builtin_vars: &BuiltinVars) {
        for (ident, _value) in &builtin_vars.values {
            self.add_symbol(ident.clone(), None);
        }
    }
}

impl<'a> Visitor<'a> for Resolver<'a> {
    fn visit_expr(&mut self, expr: &'a Expr) {
        walk_expr(self, expr);

        match expr {
            Expr::Identifier(ident) => {
                let symbol = self.resolve_symbol(ident, 0..0);
                if let Some((offset, symbol)) = symbol {
                    self.resolved_symbol_table.insert(
                        expr as *const Expr,
                        ResolvedSymbol {
                            offset: offset as i32,
                            is_upvalue: symbol.borrow().is_captured,
                        },
                    );
                }
            }
            Expr::FnCall { ident, args } => {
                for expr in args {
                    self.visit_expr(expr);
                }

                let symbol = self.resolve_symbol(ident, 0..0);
                if let Some((offset, symbol)) = symbol {
                    self.resolved_symbol_table.insert(
                        expr as *const Expr,
                        ResolvedSymbol {
                            offset: offset as i32,
                            is_upvalue: symbol.borrow().is_captured,
                        },
                    );
                }
            }
            _ => {}
        }
    }

    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        // Do not use default walking logic.

        match stmt {
            Stmt::LetDeclaration {
                meta: _,
                ident,
                initializer,
            } => {
                self.visit_expr(initializer);
                self.add_symbol(ident.clone(), Some(stmt));
            }
            Stmt::FnDeclaration {
                meta: _,
                ident,
                params,
                body,
            } => {
                self.add_symbol(ident.clone(), Some(stmt)); // Add symbol first to allow for recursion.

                let old_func_offset = self.current_func_offset;
                let old_upvalues = mem::take(&mut self.current_upvalues);

                self.current_func_offset = self.accessible_symbols.len() as i32;

                self.enter_scope();
                // add arguments
                for param in params {
                    self.add_symbol(param.clone(), Some(stmt));
                }

                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.exit_scope();

                // patch self.symbol_table with upvalues
                self.symbol_table.get(&(stmt as *const Stmt)).unwrap().borrow_mut().upvalues = mem::take(&mut self.current_upvalues);

                self.current_func_offset = old_func_offset;
                self.current_upvalues = old_upvalues;
            }
            Stmt::Block(body) => {
                self.enter_scope();
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.exit_scope();
            }
            Stmt::ExprStmt(expr) => self.visit_expr(expr),
            Stmt::ReturnStmt(expr) => self.visit_expr(expr),
            Stmt::Error => {}
        }
    }
}
