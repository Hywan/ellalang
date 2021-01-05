use crate::{
    chunk::{Chunk, OpCode},
    value::{
        object::{Obj, ObjKind},
        Value, ValueArray,
    },
};
use num_traits::FromPrimitive;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    RuntimeError { message: String, line: usize },
}

struct CallFrame<'a> {
    /// Instruction pointer.
    ip: usize,
    chunk: &'a Chunk,
}

pub struct Vm<'a> {
    /// VM stack.
    stack: ValueArray,
    call_stack: Vec<CallFrame<'a>>,
}

impl<'a> Vm<'a> {
    fn chunk(&self) -> &Chunk {
        &self.call_stack.last().unwrap().chunk
    }

    fn code(&self) -> &[u8] {
        &self.call_stack.last().unwrap().chunk.code
    }

    fn ip_mut(&mut self) -> &mut usize {
        &mut self.call_stack.last_mut().unwrap().ip
    }

    fn ip(&self) -> usize {
        self.call_stack.last().unwrap().ip
    }

    fn read_byte(&mut self) -> u8 {
        let instr = self.code()[self.ip()];
        *self.ip_mut() += 1;
        instr
    }

    fn read_constant(&mut self) -> Value {
        let constant = self.chunk().constants[self.code()[self.ip()] as usize].clone();
        *self.ip_mut() += 1;
        constant
    }

    fn runtime_error(&self, message: impl ToString) -> InterpretResult {
        InterpretResult::RuntimeError {
            message: message.to_string(),
            line: self.chunk().lines[self.ip() - 1], // -1 to get the last instruction
        }
    }

    fn run(&mut self) -> InterpretResult {
        /// Generate vm for binary operator.
        macro_rules! gen_num_binary_op {
            ($self: ident, $op: tt, $result: path) => {{
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

                $self.stack.push($result(a $op b));
            }};

            ($self: ident, $op: tt) => {
                gen_num_binary_op!($self, $op, $crate::value::Value::Number)
            }
        }

        while self.ip() < self.code().len() {
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
                Some(OpCode::Add) => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    let a_num = a.cast_to_number();
                    let b_num = b.cast_to_number();
                    if a_num.is_some() && b_num.is_some() {
                        self.stack
                            .push(Value::Number(a_num.unwrap() + b_num.unwrap()));
                    } else {
                        let a_str = a.cast_to_str();
                        let b_str = b.cast_to_str();
                        if a_str.is_some() && b_str.is_some() {
                            // handle string concatenation
                            let obj = Rc::new(Obj::new_string(format!(
                                "{}{}",
                                a_str.unwrap(),
                                b_str.unwrap()
                            )));
                            self.stack.push(Value::Object(obj));
                        } else {
                            return self.runtime_error("Operands must be numbers or strings.");
                        }
                    }
                }
                Some(OpCode::Sub) => gen_num_binary_op!(self, -),
                Some(OpCode::Mul) => gen_num_binary_op!(self, *),
                Some(OpCode::Div) => gen_num_binary_op!(self, /),
                Some(OpCode::Ret) => {
                    if self.call_stack.len() <= 1 {
                        return self.runtime_error("Can only use return in a function.");
                    }
                    self.call_stack.pop().unwrap(); // remove a `CallFrame` from the call stack.
                }
                Some(OpCode::LdTrue) => self.stack.push(Value::Bool(true)),
                Some(OpCode::LdFalse) => self.stack.push(Value::Bool(false)),
                Some(OpCode::Eq) => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b));
                }
                Some(OpCode::Greater) => gen_num_binary_op!(self, >, Value::Bool),
                Some(OpCode::Less) => gen_num_binary_op!(self, <, Value::Bool),
                Some(OpCode::Pop) => {
                    self.stack.pop().unwrap(); // throw away result
                }
                Some(OpCode::Calli) => {
                    match self.stack.pop().unwrap() {
                        Value::Object(obj) => match obj.kind {
                            ObjKind::Fn { arity, .. } => {
                                // execute the function
                                let calli_arity = self.read_byte();

                                if arity != calli_arity as u32 {
                                    return self.runtime_error(format!(
                                        "Expected {} arguments, received {}.",
                                        arity, calli_arity
                                    ));
                                }

                                for _i in 0..calli_arity {
                                    self.stack.pop().unwrap(); // arguments
                                }
                            }
                            _ => return self.runtime_error("Value is not a function."),
                        },
                        _ => return self.runtime_error("Value is not a function."),
                    }
                }
                None => panic!("Invalid instruction"),
            }

            eprintln!("IP: {}, VM stack: {:?}", self.ip(), self.stack);
        }

        InterpretResult::Ok
    }

    /// Executes the chunk
    pub fn interpret(chunk: Chunk) -> InterpretResult {
        let mut vm = Vm {
            stack: Vec::with_capacity(256),
            call_stack: vec![CallFrame {
                chunk: &chunk, // global chunk
                ip: 0,         // start interpreting at first opcode
            }],
        };
        vm.run()
    }
}
