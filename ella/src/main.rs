use ella_parser::parser::Parser;
use ella_vm::{chunk::OpCode, codegen::Codegen, vm::Vm};
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
            let mut codegen = Codegen::new();
            codegen.codegen_function(&mut ast);
            let mut chunk = codegen.into_inner_chunk();
            chunk.write_chunk(OpCode::Calli, 0);
            chunk.write_chunk(0, 0);
            eprintln!("{}", chunk);

            eprintln!("{:?}", Vm::interpret(chunk));
        }
    }
}
