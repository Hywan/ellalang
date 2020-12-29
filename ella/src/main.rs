use ella_parser::parser::Parser;
use std::io::{self, Write};

fn main() {
    // let source = r"fn foo(x) { return x * x; } ";

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
