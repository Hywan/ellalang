use ella_value::chunk::{Chunk, OpCode};
use ella_value::object::{NativeFn, Obj, ObjKind};
use ella_value::{Value, ValueArray};
use num_traits::FromPrimitive;

use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    RuntimeError { message: String, line: usize },
}

struct CallFrame {
    /// Instruction pointer.
    ip: usize,
    chunk: Chunk,
    /// NOTE: not actually a pointer but rather an index to the start of the `CallFrame`.
    frame_pointer: usize,
}

pub struct Vm<'a> {
    /// VM stack.
    stack: ValueArray,
    call_stack: Vec<CallFrame>,
    phantom: PhantomData<&'a ()>,
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

    fn runtime_error(&self, message: impl ToString) -> InterpretResult {
        InterpretResult::RuntimeError {
            message: message.to_string(),
            line: self.chunk().lines[self.ip() - 1], // -1 to get the last instruction
        }
    }

    fn run(&mut self) -> InterpretResult {
        macro_rules! read_byte {
            () => {{
                let instr: u8 = self.code()[self.ip()];
                *self.ip_mut() += 1;
                instr
            }};
        }

        macro_rules! read_constant {
            () => {{
                let constant: Value =
                    self.chunk().constants[self.code()[self.ip()] as usize].clone();
                *self.ip_mut() += 1;
                constant
            }};
        }

        macro_rules! frame {
            () => {
                self.call_stack.last().unwrap()
            };
        }

        /// Generate vm for binary operator.
        macro_rules! gen_num_binary_op {
            ($op: tt, $result: path) => {{
                let b: Value = self.stack.pop().unwrap();
                let a: Value = self.stack.pop().unwrap();

                let a = match a {
                    Value::Number(val) => val,
                    _ => return self.runtime_error("Operands must be numbers."),
                };

                let b = match b {
                    Value::Number(val) => val,
                    _ => return self.runtime_error("Operands must be numbers."),
                };

                self.stack.push($result(a $op b));
            }};

            ($op: tt) => {
                gen_num_binary_op!($op, Value::Number)
            }
        }

        while self.ip() < self.code().len() {
            match OpCode::from_u8(read_byte!()) {
                Some(OpCode::Ldc) => {
                    let constant = read_constant!();
                    self.stack.push(constant);
                }
                Some(OpCode::LdLoc) => {
                    let local_index = read_byte!() + frame!().frame_pointer as u8;
                    let local = self.stack[local_index as usize].clone();
                    self.stack.push(local);
                }
                Some(OpCode::StLoc) => {
                    let local_index = read_byte!() + frame!().frame_pointer as u8;
                    let value = self.stack.pop().unwrap();
                    self.stack[local_index as usize] = value;
                }
                Some(OpCode::LdArg) => {
                    let local_index = read_byte!() + frame!().frame_pointer as u8;
                    let local = self.stack[local_index as usize].clone();
                    self.stack.push(local);
                }
                Some(OpCode::StArg) => {
                    let local_index = read_byte!() + frame!().frame_pointer as u8;
                    let value = self.stack.pop().unwrap();
                    self.stack[local_index as usize] = value;
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
                Some(OpCode::Sub) => gen_num_binary_op!(-),
                Some(OpCode::Mul) => gen_num_binary_op!(*),
                Some(OpCode::Div) => gen_num_binary_op!(/),
                Some(OpCode::Ret) => {
                    if self.call_stack.len() <= 1 {
                        return self.runtime_error("Can only use return in a function.");
                    }
                    let return_value = self.stack.pop().unwrap();
                    let frame = self.call_stack.pop().unwrap(); // remove a `CallFrame` from the call stack.
                                                                // cleanup local variables created in function
                    while self.stack.len() > frame.frame_pointer {
                        self.stack.pop().unwrap();
                    }
                    self.stack.push(return_value);
                }
                Some(OpCode::LdTrue) => self.stack.push(Value::Bool(true)),
                Some(OpCode::LdFalse) => self.stack.push(Value::Bool(false)),
                Some(OpCode::Eq) => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(a == b));
                }
                Some(OpCode::Greater) => gen_num_binary_op!(>, Value::Bool),
                Some(OpCode::Less) => gen_num_binary_op!(<, Value::Bool),
                Some(OpCode::Pop) => {
                    self.stack.pop().unwrap(); // throw away result
                }
                Some(OpCode::Calli) => {
                    match self.stack.pop().unwrap() {
                        Value::Object(obj) => match obj.kind {
                            ObjKind::Fn {
                                ident: _,
                                arity,
                                ref chunk,
                            } => {
                                let calli_arity = read_byte!();

                                if arity != calli_arity as u32 {
                                    return self.runtime_error(format!(
                                        "Expected {} argument(s), received {}.",
                                        arity, calli_arity
                                    ));
                                }

                                // add new `CallFrame` to call stack
                                self.call_stack.push(CallFrame {
                                    ip: 0,
                                    chunk: chunk.clone(),
                                    frame_pointer: self.stack.len() - arity as usize,
                                });
                            }
                            ObjKind::NativeFn(NativeFn {
                                ident: _,
                                arity,
                                ref func,
                            }) => {
                                let calli_arity = read_byte!();

                                if arity != calli_arity as u32 {
                                    return self.runtime_error(format!(
                                        "Expected {} argument(s), received {}.",
                                        arity, calli_arity
                                    ));
                                }

                                let stack_len = self.stack.len();
                                let result = func(
                                    &mut self.stack[stack_len - 1 - arity as usize..stack_len - 1],
                                );
                                // remove arguments from stack
                                for _i in 0..arity {
                                    self.stack.pop().unwrap();
                                }
                                self.stack.push(result);
                            }
                            _ => return self.runtime_error("Value is not a function."),
                        },
                        _ => return self.runtime_error("Value is not a function."),
                    }
                }
                None => panic!("Invalid instruction"),
            }

            eprintln!(
                "IP: {ip}, Chunk: {chunk}, VM stack: {stack:?}",
                ip = self.ip(),
                chunk = self.chunk().name,
                stack = self.stack
            );
        }

        InterpretResult::Ok
    }

    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            call_stack: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Executes the chunk
    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        self.call_stack.push(CallFrame {
            ip: 0,            // start interpreting at first opcode
            chunk,            // global chunk
            frame_pointer: 0, // global frame_pointer points to start of stack
        });

        self.run()
    }

    pub fn stack(&self) -> &ValueArray {
        &self.stack
    }

    pub fn restore_stack(&mut self, stack: ValueArray) {
        self.stack = stack;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_call_frame() {
        let cf = CallFrame {
            chunk: Chunk::new("test".to_string()),
            ip: 0,
            frame_pointer: 0,
        };

        assert_eq!(cf.ip, 0);
        assert_eq!(cf.frame_pointer, 0);
    }
}
