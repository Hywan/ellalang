use ella_parser::parser::Parser;
use ella_passes::resolve::Resolver;
use ella_vm::vm::InterpretResult;
use ella_vm::{codegen::Codegen, vm::Vm};
use std::io::{self, Write};

fn main() {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    let mut vm = Vm::new();
    let mut resolved_symbols = Vec::new();
    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let source = input.as_str().into();
        let mut parser = Parser::new(&source);
        let mut ast = parser.parse_program();
        // eprintln!("{:#?}", ast);

        let mut resolver = Resolver::new_with_existing_symbols(&source, resolved_symbols.clone());
        resolver.resolve_top_level(&mut ast);
        let resolved_symbol_table = resolver.resolved_symbol_table();

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
