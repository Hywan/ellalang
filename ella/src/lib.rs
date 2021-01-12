pub mod builtin_functions;

/// For testing purposes only.
pub fn interpret(source: &str) {
    use builtin_functions::default_builtin_vars;

    use ella_parser::parser::Parser;
    use ella_passes::resolve::Resolver;
    use ella_vm::codegen::Codegen;
    use ella_vm::vm::{InterpretResult, Vm};

    let builtin_vars = default_builtin_vars();

    let dummy_source = "".into();
    let mut resolver = Resolver::new(&dummy_source);
    resolver.resolve_builtin_vars(&builtin_vars);
    let mut resolve_result = resolver.resolve_result();
    let accessible_symbols = resolver.accessible_symbols();

    let mut vm = Vm::new(&builtin_vars);
    let mut codegen = Codegen::new("<global>".to_string(), resolve_result);
    codegen.codegen_builtin_vars(&builtin_vars);
    vm.interpret(codegen.into_inner_chunk()); // load built in functions into memory

    let source = source.into();
    let mut parser = Parser::new(&source);
    let ast = parser.parse_program();

    let mut resolver =
        Resolver::new_with_existing_accessible_symbols(&source, accessible_symbols.clone());
    resolver.resolve_top_level(&ast);
    resolve_result = resolver.resolve_result();

    eprintln!("{}", source.errors);
    assert!(source.has_no_errors());

    let mut codegen = Codegen::new("<global>".to_string(), resolve_result);

    codegen.codegen_function(&ast);

    let chunk = codegen.into_inner_chunk();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}
