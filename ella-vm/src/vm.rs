use crate::{
    chunk::{Chunk, OpCode},
    value::{object::Obj, Value, ValueArray},
};
use num_traits::FromPrimitive;
use std::ptr;

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
    /// A single linked list to all the objects.
    /// When allocating new objects, the `Box` should be leaked and added to the head of this field.
    /// When the VM is dropped, all the objects in this field should be dropped as well.
    objects: *mut Obj,
}

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

impl Vm {
    fn read_byte(&mut self) -> u8 {
        let instr = self.chunk.code[self.ip];
        self.ip += 1;
        instr
    }

    fn read_constant(&mut self) -> Value {
        let constant = self.chunk.constants[self.chunk.code[self.ip] as usize].clone();
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
                            self.stack
                                .push(Value::Object(Box::new(Obj::new_string(format!(
                                    "{}{}",
                                    a_str.unwrap(),
                                    b_str.unwrap()
                                )))));
                        } else {
                            return self.runtime_error("Operands must be numbers or strings.");
                        }
                    }
                }
                Some(OpCode::Sub) => gen_num_binary_op!(self, -),
                Some(OpCode::Mul) => gen_num_binary_op!(self, *),
                Some(OpCode::Div) => gen_num_binary_op!(self, /),
                Some(OpCode::Ret) => {
                    println!("{}", self.stack.pop().unwrap()); // return value
                    return InterpretResult::Ok;
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
            objects: ptr::null_mut(),
        };
        vm.run()
    }
}

impl Drop for Vm {
    fn drop(&mut self) {
        let mut object = self.objects;
        while !object.is_null() {
            let next = unsafe { ptr::read(self.objects).next };
            eprintln!("Dropping object {:?}", object);
            unsafe { ptr::drop_in_place(object) };
            object = next;
        }
    }
}
