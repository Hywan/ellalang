use ella_parser::parser::Parser;
use ella_vm::{codegen::Codegen, vm::Vm};
use std::io::{self, Write};

fn main() {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let source = input.as_str().into();
        let mut parser = Parser::new(&source);
        let mut ast = parser.parse_program();
        // eprintln!("{:#?}", ast);

        eprintln!("{}", source.errors);
        if source.has_no_errors() {
            let mut codegen = Codegen::new("<global>".to_string());
            codegen.codegen_function(&mut ast);
            let chunk = codegen.into_inner_chunk();

            eprintln!("{:?}", Vm::interpret(chunk));
        }
    }
}
