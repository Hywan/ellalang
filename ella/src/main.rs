use ella_parser::parser::Parser;
use ella_passes::resolve::Resolver;
use ella_value::BuiltinVars;
use ella_vm::vm::InterpretResult;
use ella_vm::{codegen::Codegen, vm::Vm};
use std::collections::HashMap;
use std::io::{self, Write};

mod builtin_functions;

fn repl() {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    let builtin_vars = {
        let mut builtin_vars = BuiltinVars::new();
        builtin_vars.add_native_fn("print", &builtin_functions::print, 1);
        builtin_vars.add_native_fn("println", &builtin_functions::println, 1);
        builtin_vars.add_native_fn("assert_eq", &builtin_functions::assert_eq, 2);
        builtin_vars.add_native_fn("assert", &builtin_functions::assert, 1);
        builtin_vars.add_native_fn("clock", &builtin_functions::clock, 0);
        builtin_vars
    };

    let mut resolved_symbols = {
        let dummy_source = "".into();
        let mut resolver = Resolver::new(&dummy_source);
        resolver.resolve_builtin_vars(&builtin_vars);
        resolver.into_resolved_symbols()
    };

    let mut vm = Vm::new(&builtin_vars);
    let mut resolved_symbol_table = &HashMap::new();
    let mut codegen = Codegen::new("<global>".to_string(), &resolved_symbol_table);
    codegen.codegen_builtin_vars(&builtin_vars);
    vm.interpret(codegen.into_inner_chunk()); // load built in functions into memory

    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let source = input.as_str().into();
        let mut parser = Parser::new(&source);
        let mut ast = parser.parse_repl_input();
        // eprintln!("{:#?}", ast);

        let mut resolver = Resolver::new_with_existing_symbols(&source, resolved_symbols.clone());
        resolver.resolve_top_level(&mut ast);
        resolved_symbol_table = resolver.resolved_symbol_table();

        eprintln!("{}", source.errors);
        if source.has_no_errors() {
            let mut codegen = Codegen::new("<global>".to_string(), resolved_symbol_table);

            codegen.codegen_function(&mut ast);

            let chunk = codegen.into_inner_chunk();

            let initial_stack = vm.stack().clone();
            let interpret_result = vm.interpret(chunk);
            match &interpret_result {
                InterpretResult::Ok => {
                    // Success, update  resolved_symbols with new symbols.
                    resolved_symbols = resolver.into_resolved_symbols();
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
    let builtin_vars = {
        let mut builtin_vars = BuiltinVars::new();
        builtin_vars.add_native_fn("print", &builtin_functions::print, 1);
        builtin_vars.add_native_fn("println", &builtin_functions::println, 1);
        builtin_vars.add_native_fn("assert_eq", &builtin_functions::assert_eq, 2);
        builtin_vars.add_native_fn("assert", &builtin_functions::assert, 1);
        builtin_vars.add_native_fn("clock", &builtin_functions::clock, 0);
        builtin_vars
    };

    let resolved_symbols = {
        let dummy_source = "".into();
        let mut resolver = Resolver::new(&dummy_source);
        resolver.resolve_builtin_vars(&builtin_vars);
        resolver.into_resolved_symbols()
    };

    let mut vm = Vm::new(&builtin_vars);
    let mut resolved_symbol_table = &HashMap::new();
    let mut codegen = Codegen::new("<global>".to_string(), &resolved_symbol_table);
    codegen.codegen_builtin_vars(&builtin_vars);
    vm.interpret(codegen.into_inner_chunk()); // load built in functions into memory

    let source = source.into();
    let mut parser = Parser::new(&source);
    let mut ast = parser.parse_program();

    let mut resolver = Resolver::new_with_existing_symbols(&source, resolved_symbols.clone());
    resolver.resolve_top_level(&mut ast);
    resolved_symbol_table = resolver.resolved_symbol_table();

    if !source.has_no_errors() {
        eprintln!("{}", source.errors);
    } else {
        let mut codegen = Codegen::new("<global>".to_string(), resolved_symbol_table);

        codegen.codegen_function(&mut ast);

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
