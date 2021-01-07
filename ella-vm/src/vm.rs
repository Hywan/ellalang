use ella_value::chunk::{Chunk, OpCode};
use ella_value::object::{Closure, Function, NativeFn, Obj, ObjKind, UpValue};
use ella_value::{BuiltinVars, Value, ValueArray};
use num_traits::FromPrimitive;

use std::cell::RefCell;
use std::rc::Rc;

const INSPECT_VM_STACK: bool = false;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpretResult {
    Ok,
    RuntimeError { message: String, line: usize },
}

#[derive(Clone)]
struct CallFrame {
    /// Instruction pointer.
    ip: usize,
    /// NOTE: not actually a pointer but rather an index to the start of the `CallFrame`.
    frame_pointer: usize,
    closure: Rc<Closure>,
}

pub struct Vm<'a> {
    /// VM stack.
    stack: ValueArray,
    call_stack: Vec<CallFrame>,
    builtin_vars: &'a BuiltinVars,
    upvalues: Vec<Rc<RefCell<UpValue>>>,
}

impl<'a> Vm<'a> {
    fn chunk(&self) -> &Chunk {
        &self.call_stack.last().unwrap().closure.func.chunk
    }

    fn code(&self) -> &[u8] {
        &self.call_stack.last().unwrap().closure.func.chunk.code
    }

    fn ip_mut(&mut self) -> &mut usize {
        &mut self.call_stack.last_mut().unwrap().ip
    }

    fn ip(&self) -> usize {
        self.call_stack.last().unwrap().ip
    }

    fn resolve_upvalue_into_value(&self, upvalue: &UpValue) -> Value {
        match upvalue {
            UpValue::Open(index) => self.stack[*index].clone(),
            UpValue::Closed(value) => value.clone(),
        }
    }

    fn set_upvalue(&mut self, upvalue: Rc<RefCell<UpValue>>, new_value: Value) {
        match *upvalue.borrow_mut() {
            UpValue::Open(index) => self.stack[index] = new_value,
            UpValue::Closed(ref _value) => *upvalue.borrow_mut() = UpValue::Closed(new_value),
        }
    }

    fn close_upvalues(&mut self, index: usize) {
        let value = self.stack[index].clone();
        for upvalue in &self.upvalues {
            if upvalue.borrow().is_open_with_index(index) {
                *upvalue.borrow_mut() = UpValue::Closed(value.clone());
            }
        }
    }

    fn find_open_upvalue_with_index(&self, index: usize) -> Option<Rc<RefCell<UpValue>>> {
        for upvalue in &self.upvalues {
            if upvalue.borrow().is_open_with_index(index) {
                return Some(upvalue.clone());
            }
        }
        None
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
                let byte: u8 = self.code()[self.ip()];
                *self.ip_mut() += 1;
                byte
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

        /// Uses the last value on the stack as the return value and cleans up the local variables created inside the function.
        macro_rules! cleanup_function {
            () => {{
                let return_value = self.stack.pop().unwrap();
                let frame = self.call_stack.pop().unwrap(); // remove a `CallFrame` from the call stack.
                                                            // cleanup local variables created in function
                while self.stack.len() > frame.frame_pointer {
                    self.stack.pop().unwrap();
                }
                self.stack.push(return_value);
            }}
        }

        /// If inside a function, cleans up and returns `true`. Else returns `false` and does nothing.
        macro_rules! try_implicit_ret {
            () => {{
                if self.call_stack.len() > 1 {
                    // inside a function
                    self.stack.push(Value::Number(0.0)); // FIXME: returns 0.0 by default
                    cleanup_function!();
                    true
                } else {
                    self.call_stack.pop().unwrap();
                    false
                }
            }};
        }

        while self.ip() < self.code().len() || try_implicit_ret!() {
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
                Some(OpCode::LdUpVal) => {
                    let index = read_byte!();
                    let upvalue =
                        self.call_stack.last().unwrap().closure.upvalues[index as usize].clone();
                    let value = self.resolve_upvalue_into_value(&upvalue.borrow());
                    self.stack.push(value);
                }
                Some(OpCode::StUpVal) => {
                    let index = read_byte!();
                    let value = self.stack.pop().unwrap();
                    let upvalue =
                        self.call_stack.last().unwrap().closure.upvalues[index as usize].clone();
                    self.set_upvalue(upvalue, value);
                }
                Some(OpCode::CloseUpVal) => {
                    let index = self.stack.len() - 1;
                    self.close_upvalues(index);
                    self.stack.pop().unwrap();
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
                    cleanup_function!();
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
                        Value::Object(obj) => match &obj.kind {
                            // ObjKind::Fn(Function {
                            //     ident: _,
                            //     arity,
                            //     ref chunk,
                            // }) => {
                            //     let calli_arity = read_byte!();

                            //     if arity != calli_arity as u32 {
                            //         return self.runtime_error(format!(
                            //             "Expected {} argument(s), received {}.",
                            //             arity, calli_arity
                            //         ));
                            //     }

                            //     // add new `CallFrame` to call stack
                            //     self.call_stack.push(CallFrame {
                            //         ip: 0,
                            //         chunk: chunk.clone(),
                            //         frame_pointer: self.stack.len() - arity as usize,
                            //     });
                            // }
                            ObjKind::Closure(closure) => {
                                let calli_arity = read_byte!();

                                if closure.func.arity != calli_arity as u32 {
                                    return self.runtime_error(format!(
                                        "Expected {} argument(s), received {}.",
                                        closure.func.arity, calli_arity
                                    ));
                                }

                                // add new `CallFrame` to call stack
                                self.call_stack.push(CallFrame {
                                    ip: 0,
                                    frame_pointer: self.stack.len() - closure.func.arity as usize,
                                    closure: Rc::new(closure.clone()),
                                });
                            }
                            ObjKind::NativeFn(NativeFn {
                                ident: _,
                                arity,
                                func,
                            }) => {
                                let calli_arity = read_byte!();

                                if *arity != calli_arity as u32 {
                                    return self.runtime_error(format!(
                                        "Expected {} argument(s), received {}.",
                                        arity, calli_arity
                                    ));
                                }

                                let stack_len = self.stack.len();
                                let args = &mut self.stack[stack_len - *arity as usize..stack_len];
                                debug_assert_eq!(args.len(), *arity as usize);

                                let result = func(args);
                                // remove arguments from stack
                                for _i in 0..*arity {
                                    self.stack.pop().unwrap();
                                }
                                self.stack.push(result);
                            }
                            _ => return self.runtime_error("Value is not a function."),
                        },
                        _ => return self.runtime_error("Value is not a function."),
                    }
                }
                Some(OpCode::Closure) => {
                    let func = match read_constant!() {
                        Value::Object(obj) => match &obj.kind {
                            ObjKind::Fn(function) => function.clone(),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    };

                    let upvalues_count = func.upvalues_count;
                    let mut closure = Closure {
                        func,
                        upvalues: Vec::with_capacity(upvalues_count),
                    };

                    for _i in 0..upvalues_count {
                        let _is_local = read_byte!();
                        let index = read_byte!() + frame!().frame_pointer as u8;

                        let upvalue = match self.find_open_upvalue_with_index(index as usize) {
                            Some(upvalue) => upvalue,
                            None => {
                                let upvalue = Rc::new(RefCell::new(UpValue::Open(index as usize)));
                                self.upvalues.push(upvalue.clone());
                                upvalue
                            }
                        };

                        closure.upvalues.push(upvalue);
                    }
                    debug_assert_eq!(closure.upvalues.len(), upvalues_count);

                    self.stack.push(Value::Object(Rc::new(Obj {
                        kind: ObjKind::Closure(closure),
                    })));
                }
                None => panic!("Invalid instruction"),
            }

            if INSPECT_VM_STACK {
                eprintln!(
                    "IP: {ip}, Chunk: {chunk}, VM stack: {stack:?}",
                    ip = self.ip(),
                    chunk = self.chunk().name,
                    stack = &self.stack[(self.builtin_vars.values.len()).min(self.stack.len())..] // do not show builtin vars in stack
                );
            }
        }

        InterpretResult::Ok
    }

    pub fn new(builtin_vars: &'a BuiltinVars) -> Self {
        Self {
            stack: Vec::new(),
            call_stack: Vec::new(),
            builtin_vars,
            upvalues: Vec::new(),
        }
    }

    /// Executes the chunk
    pub fn interpret(&mut self, chunk: Chunk) -> InterpretResult {
        let func = Function {
            arity: 0,
            chunk,
            ident: "top".to_string(),
            upvalues_count: 0, // cannot have any upvalues for top-level function
        };
        let closure = Closure {
            func,
            upvalues: Vec::new(),
        };
        self.call_stack.push(CallFrame {
            ip: 0,            // start interpreting at first opcode
            frame_pointer: 0, // global frame_pointer points to start of stack
            closure: Rc::new(closure),
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
