use ella_parser::parser::Parser;
use ella_parser::visitor::Visitor;
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
        eprintln!("{:#?}", ast);

        eprintln!("{}", source.errors);
        if source.has_no_errors() {
            let mut codegen = Codegen::new();
            codegen.visit_expr(&mut ast);
            let mut chunk = codegen.into_inner_chunk();
            eprintln!("{}", chunk);

            chunk.write_chunk(ella_vm::chunk::OpCode::Ret, 0);
            eprintln!("{:?}", Vm::interpret(chunk));
        }
    }
}
