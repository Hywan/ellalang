use ella::builtin_functions::default_builtin_vars;
use ella_parser::parser::Parser;
use ella_passes::resolve::{ResolvedSymbolTable, Resolver};
use ella_vm::vm::InterpretResult;
use ella_vm::{codegen::Codegen, vm::Vm};

use std::io::{self, Write};

mod builtin_functions;

fn repl() {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    let builtin_vars = default_builtin_vars();

    let dummy_source = "".into();
    let mut resolver = Resolver::new(&dummy_source);
    resolver.resolve_builtin_vars(&builtin_vars);
    let mut symbol_table = resolver.symbol_table().clone();
    let mut accessible_symbols = resolver.accessible_symbols().clone();

    let mut vm = Vm::new(&builtin_vars);
    let mut resolved_symbol_table = &ResolvedSymbolTable::new();
    let mut codegen = Codegen::new("<global>".to_string(), &symbol_table, resolved_symbol_table);
    codegen.codegen_builtin_vars(&builtin_vars);
    vm.interpret(codegen.into_inner_chunk()); // load built in functions into memory

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let source = input.as_str().into();
        let mut parser = Parser::new(&source);
        let ast = parser.parse_repl_input();

        let mut resolver =
            Resolver::new_with_existing_accessible_symbols(&source, accessible_symbols.clone());
        resolver.resolve_top_level(&ast);
        symbol_table = resolver.symbol_table().clone();
        resolved_symbol_table = resolver.resolved_symbol_table();

        eprintln!("{}", source.errors);
        if source.has_no_errors() {
            let mut codegen =
                Codegen::new("<global>".to_string(), &symbol_table, resolved_symbol_table);

            codegen.codegen_function(&ast);

            let chunk = codegen.into_inner_chunk();

            let initial_stack = vm.stack().clone();
            let interpret_result = vm.interpret(chunk);
            match &interpret_result {
                InterpretResult::Ok => {
                    // Success, update  resolved_symbols with new symbols.
                    accessible_symbols = resolver.accessible_symbols().clone();
                }
                InterpretResult::RuntimeError { .. } => {
                    eprintln!("{:?}", interpret_result);
                    // Restore vm stack to previous state to recover from error.
                    vm.restore_stack(initial_stack);
                }
            }
        }
    }
}

fn interpret_file_contents(source: &str) {
    let builtin_vars = default_builtin_vars();

    let dummy_source = "".into();
    let mut resolver = Resolver::new(&dummy_source);
    resolver.resolve_builtin_vars(&builtin_vars);
    let mut symbol_table = resolver.symbol_table();
    let accessible_symbols = resolver.accessible_symbols();

    let mut vm = Vm::new(&builtin_vars);
    let mut resolved_symbol_table = &ResolvedSymbolTable::new();
    let mut codegen = Codegen::new("<global>".to_string(), symbol_table, resolved_symbol_table);
    codegen.codegen_builtin_vars(&builtin_vars);
    vm.interpret(codegen.into_inner_chunk()); // load built in functions into memory

    let source = source.into();
    let mut parser = Parser::new(&source);
    let ast = parser.parse_program();

    let mut resolver =
        Resolver::new_with_existing_accessible_symbols(&source, accessible_symbols.clone());
    resolver.resolve_top_level(&ast);
    symbol_table = resolver.symbol_table();
    resolved_symbol_table = resolver.resolved_symbol_table();

    if !source.has_no_errors() {
        eprintln!("{}", source.errors);
    } else {
        let mut codegen = Codegen::new("<global>".to_string(), symbol_table, resolved_symbol_table);

        codegen.codegen_function(&ast);

        let chunk = codegen.into_inner_chunk();
        match vm.interpret(chunk) {
            InterpretResult::Ok => {}
            InterpretResult::RuntimeError { message, line } => {
                eprintln!("Runtime Error: {} at line {}", message, line);
            }
        }
    }
}

fn main() {
    if std::env::args().len() < 2 {
        repl();
    } else {
        let path = std::env::args().nth(1).unwrap();
        let contents = std::fs::read_to_string(path);
        match contents {
            Ok(contents) => interpret_file_contents(&contents),
            Err(err) => eprintln!("Error: {}", err),
        }
    }
}
