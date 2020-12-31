use crate::{
    chunk::{Chunk, OpCode},
    value::{Value, ValueArray},
};
use num_traits::FromPrimitive;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    RuntimeError { message: String, line: usize },
}

pub struct Vm {
    /// Instruction pointer.
    ip: usize,
    chunk: Chunk,
    /// VM stack.
    stack: ValueArray,
}

/// Generate vm for binary operator.
macro_rules! gen_binary_op {
    ($self: ident, $op: tt) => {{
        let b: $crate::value::Value = $self.stack.pop().unwrap();
        let a: $crate::value::Value = $self.stack.pop().unwrap();

        let a = match a {
            $crate::value::Value::Number(val) => val,
            _ => return $self.runtime_error("Operands must be numbers."),
        };

        let b = match b {
            $crate::value::Value::Number(val) => val,
            _ => return $self.runtime_error("Operands must be numbers."),
        };

        $self.stack.push($crate::value::Value::Number(a $op b));
    }};
}

impl Vm {
    fn read_byte(&mut self) -> u8 {
        let instr = self.chunk.code[self.ip];
        self.ip += 1;
        instr
    }

    fn read_constant(&mut self) -> Value {
        let constant = self.chunk.constants[self.chunk.code[self.ip] as usize];
        self.ip += 1;
        constant
    }

    fn runtime_error(&self, message: impl ToString) -> InterpretResult {
        InterpretResult::RuntimeError {
            message: message.to_string(),
            line: self.chunk.lines[self.ip - 1], // -1 to get the last instruction
        }
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            eprintln!("VM stack: {:?}", self.stack);
            match OpCode::from_u8(self.read_byte()) {
                Some(OpCode::Ldc) => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }
                Some(OpCode::Neg) => {
                    let val = self.stack.pop().unwrap();
                    match val {
                        Value::Number(val) => self.stack.push(Value::Number(-val)),
                        _ => return self.runtime_error("Operand must be a number."),
                    }
                }
                Some(OpCode::Not) => {
                    let val = self.stack.pop().unwrap();
                    match val {
                        Value::Bool(val) => self.stack.push(Value::Bool(!val)),
                        _ => return self.runtime_error("Operand must be a boolean."),
                    }
                }
                Some(OpCode::Add) => gen_binary_op!(self, +),
                Some(OpCode::Sub) => gen_binary_op!(self, -),
                Some(OpCode::Mul) => gen_binary_op!(self, *),
                Some(OpCode::Div) => gen_binary_op!(self, /),
                Some(OpCode::Ret) => {
                    println!("{}", self.stack.pop().unwrap()); // return value
                    return InterpretResult::Ok;
                }
                Some(OpCode::LdTrue) => self.stack.push(Value::Bool(true)),
                Some(OpCode::LdFalse) => self.stack.push(Value::Bool(false)),
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
