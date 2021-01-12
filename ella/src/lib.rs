pub mod builtin_functions;

/// For testing purposes only.
pub fn interpret(source: &str) {
    use std::collections::HashMap;

    use ella_parser::parser::Parser;
    use ella_passes::resolve::Resolver;
    use ella_value::BuiltinVars;
    use ella_vm::codegen::Codegen;
    use ella_vm::vm::{InterpretResult, Vm};

    let builtin_vars = {
        let mut builtin_vars = BuiltinVars::new();
        builtin_vars.add_native_fn("print", &builtin_functions::print, 1);
        builtin_vars.add_native_fn("println", &builtin_functions::println, 1);
        builtin_vars.add_native_fn("assert_eq", &builtin_functions::assert_eq, 2);
        builtin_vars.add_native_fn("assert", &builtin_functions::assert, 1);
        builtin_vars.add_native_fn("clock", &builtin_functions::clock, 0);
        builtin_vars
    };

    let dummy_source = "".into();
    let mut resolver = Resolver::new(&dummy_source);
    resolver.resolve_builtin_vars(&builtin_vars);
    let mut symbol_table = resolver.symbol_table();
    let accessible_symbols = resolver.accessible_symbols();

    let mut vm = Vm::new(&builtin_vars);
    let mut resolved_symbol_table = &HashMap::new();
    let mut codegen = Codegen::new("<global>".to_string(), symbol_table, &resolved_symbol_table);
    codegen.codegen_builtin_vars(&builtin_vars);
    vm.interpret(codegen.into_inner_chunk()); // load built in functions into memory

    let source = source.into();
    let mut parser = Parser::new(&source);
    let ast = parser.parse_program();

    let mut resolver = Resolver::new_with_existing_accessible_symbols(&source, accessible_symbols.clone());
    resolver.resolve_top_level(&ast);
    symbol_table = resolver.symbol_table();
    resolved_symbol_table = resolver.resolved_symbol_table();

    eprintln!("{}", source.errors);
    assert!(source.has_no_errors());

    let mut codegen = Codegen::new("<global>".to_string(), symbol_table, resolved_symbol_table);

    codegen.codegen_function(&ast);

    let chunk = codegen.into_inner_chunk();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}
