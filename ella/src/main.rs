use ella_parser::parser::Parser;
use ella_vm::vm::Vm;
use std::io::{self, Write};

fn main() {
    // let source = r"fn foo(x) { return x * x; } ";

    let mut chunk = ella_vm::chunk::Chunk::new();
    chunk.write_chunk(ella_vm::chunk::OpCode::Ldc, 123);
    let constant = chunk.add_constant(123456.0);
    chunk.write_chunk(constant, 123);
    chunk.write_chunk(ella_vm::chunk::OpCode::Ret, 124);
    eprintln!("{}", chunk);
    eprintln!("{:?}", Vm::interpret(chunk));

    let mut stdout = io::stdout();
    let stdin = io::stdin();
    loop {
        print!("> ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let source = input.as_str().into();
        let mut parser = Parser::new(&source);
        println!("{:#?}", parser.parse_program());
    }
}
