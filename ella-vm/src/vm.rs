use crate::{
    chunk::{Chunk, OpCode},
    value::ValueArray,
};
use num_traits::FromPrimitive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct Vm {
    /// Instruction pointer.
    ip: usize,
    chunk: Chunk,
    /// VM stack.
    stack: ValueArray,
}

impl Vm {
    fn read_byte(&mut self) -> u8 {
        let instr = self.chunk.code[self.ip];
        self.ip += 1;
        instr
    }

    fn read_constant(&mut self) -> f64 {
        let constant = self.chunk.constants[self.chunk.code[self.ip] as usize];
        self.ip += 1;
        constant
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            eprintln!("VM stack: {:?}", self.stack);
            match OpCode::from_u8(self.read_byte()) {
                Some(OpCode::Ldc) => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }
                Some(OpCode::Ret) => {
                    self.stack.pop(); // return value
                    return InterpretResult::Ok;
                }
                None => panic!("Invalid instruction"),
            }
        }
    }

    /// Executes the chunk
    pub fn interpret(chunk: Chunk) -> InterpretResult {
        let mut vm = Vm {
            ip: 0, // point to first instruction in chunk.code
            chunk,
            stack: Vec::with_capacity(256),
        };
        vm.run()
    }
}
