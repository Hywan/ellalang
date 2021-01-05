//! Variable resolution pass.

use std::collections::HashMap;
use std::ops::Range;
use std::u32;

use ella_parser::ast::{Expr, Stmt};
use ella_parser::visitor::{walk_expr, Visitor};
use ella_source::{Source, SyntaxError};

/// Represents a resolved symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedSymbol {
    ident: String,
    scope_depth: u32,
}

pub type ResolvedSymbolTable = HashMap<*const Expr, i32>;

/// Variable resolution pass.
pub struct Resolver<'a> {
    /// A [`HashMap`] mapping all [`Expr::Identifier`] to variable offsets.
    variable_offsets: ResolvedSymbolTable,
    resolved_symbols: Vec<ResolvedSymbol>,
    current_scope_depth: u32,
    /// Every time a new function scope is created, `current_func_offset` should be set to `self.resolved_symbols.len()`.
    /// When exiting a function scope, the value should be reverted to previous value.
    current_func_offset: i32,
    source: &'a Source<'a>,
}

impl<'a> Resolver<'a> {
    pub fn new(source: &'a Source) -> Self {
        Self {
            variable_offsets: HashMap::new(),
            resolved_symbols: Vec::new(),
            current_scope_depth: 0,
            current_func_offset: 0,
            source,
        }
    }

    pub fn new_with_existing_symbols(
        source: &'a Source,
        resolved_symbols: Vec<ResolvedSymbol>,
    ) -> Self {
        Self {
            resolved_symbols,
            ..Self::new(source)
        }
    }

    /// Returns a [`HashMap`] mapping all [`Expr::Identifier`] to variable offsets.
    pub fn resolved_symbol_table(&self) -> &ResolvedSymbolTable {
        &self.variable_offsets
    }

    pub fn into_resolved_symbols(self) -> Vec<ResolvedSymbol> {
        self.resolved_symbols
    }

    fn enter_scope(&mut self) {
        self.current_scope_depth += 1;
    }

    fn exit_scope(&mut self) {
        self.current_scope_depth -= 1;

        // Remove all symbols in current scope.
        self.resolved_symbols = self
            .resolved_symbols
            .iter()
            .filter(|symbol| symbol.scope_depth <= self.current_scope_depth)
            .cloned()
            .collect();
    }

    fn add_symbol(&mut self, ident: String) {
        self.resolved_symbols.push(ResolvedSymbol {
            ident,
            scope_depth: self.current_scope_depth,
        });
    }

    /// Returns the offset of a resolved variable or `0` if cannot be resolved.
    ///
    /// # Params
    /// * `ident` - The identifier to resolve.
    /// * `span` - The span of the expression to resolve. This is used for error reporting in case the variable could not be resolved.
    fn resolve_symbol(&self, ident: &str, span: Range<usize>) -> i32 {
        for (i, symbol) in self.resolved_symbols.iter().enumerate().rev() {
            if symbol.ident == ident {
                return i as i32 - self.current_func_offset;
            }
        }
        self.source.errors.add_error(SyntaxError::new(
            format!("Cannot resolve symbol {}", ident),
            span,
        ));
        -1
    }

    pub fn resolve_top_level(&mut self, func: &mut Stmt) {
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

impl<'a> Visitor for Resolver<'a> {
    fn visit_expr(&mut self, expr: &mut Expr) {
        walk_expr(self, expr);

        match expr {
            Expr::Identifier(ident) => {
                let offset = self.resolve_symbol(ident, 0..0);
                self.variable_offsets.insert(expr as *const Expr, offset);
            }
            Expr::FnCall { ident, args } => {
                for expr in args {
                    self.visit_expr(expr);
                }

                let offset = self.resolve_symbol(ident, 0..0);
                self.variable_offsets.insert(expr as *const Expr, offset);
            }
            _ => {}
        }
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        // Do not use default walking logic.

        match stmt {
            Stmt::LetDeclaration { ident, initializer } => {
                self.visit_expr(initializer);
                self.add_symbol(ident.clone());
            }
            Stmt::FnDeclaration {
                ident,
                params: _,
                body,
            } => {
                self.add_symbol(ident.clone()); // Add symbol first to allow recursion.

                let old_func_offset = self.current_func_offset;
                self.current_func_offset = self.resolved_symbols.len() as i32;

                self.enter_scope();
                for stmt in body {
                    self.visit_stmt(stmt);
                }
                self.exit_scope();

                self.current_func_offset = old_func_offset;
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
