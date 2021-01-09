use std::collections::HashMap;

use ella::builtin_functions;
use ella_parser::parser::Parser;
use ella_passes::resolve::Resolver;
use ella_value::BuiltinVars;
use ella_vm::codegen::Codegen;
use ella_vm::vm::{InterpretResult, Vm};

fn interpret(source: &str) {
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
    let ast = parser.parse_program();

    let mut resolver = Resolver::new_with_existing_symbols(&source, resolved_symbols.clone());
    resolver.resolve_top_level(&ast);
    resolved_symbol_table = resolver.resolved_symbol_table();

    assert!(source.has_no_errors());

    let mut codegen = Codegen::new("<global>".to_string(), resolved_symbol_table);

    codegen.codegen_function(&ast);

    let chunk = codegen.into_inner_chunk();
    assert_eq!(vm.interpret(chunk), InterpretResult::Ok);
}

#[test]
#[should_panic]
fn smoke_assert() {
    interpret(
        r#"
        assert(false);"#,
    );
}

#[test]
#[should_panic]
fn smoke_assert_eq() {
    interpret(
        r#"
        assert_eq(1, 2);"#,
    );
}

#[test]
fn variables() {
    interpret(
        r#"
        let x = 1;
        assert_eq(x, 1);
        let y = x + 1;
        assert_eq(y, 2);
        assert_eq(y, x + 1);
        x = 10;
        assert_eq(x, 10);"#,
    );
}

#[test]
fn comments() {
    interpret(
        r#"
        let x = 1; // a comment
        assert_eq(x, 1);"#,
    );
}

mod functions {
    use super::*;

    #[test]
    fn functions() {
        interpret(
            r#"
            fn foo() {
                return 1;
            }
            assert_eq(foo(), 1);"#,
        );
    }

    #[test]
    fn functions_with_params() {
        interpret(
            r#"
            fn double(x) {
                let result = x * 2;
                return result;
            }
            assert_eq(double(10), 20);
            assert_eq(double(-2), -4);"#,
        );
    }

    #[test]
    fn functions_implicit_return() {
        interpret(
            r#"
            fn foo() { }
            assert_eq(foo(), 0);"#,
        );
    }

    #[test]
    fn higher_order_function() {
        interpret(
            r#"
            fn twice(f, v) {
                return f(f(v));
            }
            fn double(x) {
                return x * 2;
            }
            
            assert_eq(twice(double, 10), 40);
            assert_eq(twice(double, -2), -8);"#,
        );
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn closures() {
        interpret(
            r#"
            fn createAdder(x) {
                fn adder(y) {
                    return x + y;
                }
                return adder;
            }
            let addTwo = createAdder(2);
            assert_eq(addTwo(1), 10);
            assert(false);"#,
        );
        interpret(
            r#"
            fn compose(f, g) {
                function func(x) {
                    return f(g(x));
                }
                return func;
            }
            fn addOne(x) { return x + 1; }
            fn addTwo(x) { return x + 2; }
            assert_eq(compose(addOne, addTwo)(2), 5);"#,
        );
    }
}
