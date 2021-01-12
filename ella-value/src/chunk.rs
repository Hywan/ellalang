//! Definitions for [`Chunk`] and [`OpCode`].

use crate::{Value, ValueArray};
use enum_primitive_derive::Primitive;

/// Represents an opcode. Internally represented using 1 byte (`u8`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Primitive)]
#[repr(u8)]
pub enum OpCode {
    /// Load a constant onto the stack.
    /// *2 bytes (1 operand)*
    Ldc = 0,
    /// Load a local variable onto the stack.
    /// *2 bytes (1 operand)*
    LdLoc = 15,
    /// Stores the top value on the stack into a local variable.
    /// *2 bytes (1 operand)*
    StLoc = 16,
    /// Loads an upvalue onto the stack.
    /// *2 bytes (1 operand)*
    LdUpVal = 17,
    /// Stores the top value on the stack into an upvalue.
    /// *2 bytes (1 operand)*
    StUpVal = 18,
    /// Closes an upvalue.
    /// *1 byte*
    CloseUpVal = 20,
    /// Negate the last value on the stack.
    /// *1 byte*
    Neg = 1,
    /// Logical not on a boolean value.
    /// *1 byte*
    Not = 2,
    Add = 3,
    Sub = 4,
    Mul = 5,
    Div = 6,
    /// Returns the last value on the stack.
    /// *1 byte*
    Ret = 7,
    /// Loads `true` onto the stack.
    /// *1 byte*
    LdTrue = 8,
    /// Loads `false` onto the stack.
    /// *1 byte*
    LdFalse = 9,
    Eq = 10,
    Greater = 11,
    Less = 12,
    /// Pops and disposes the last value on the stack.
    /// *1 byte*
    Pop = 13,
    /// Calls the function on the top of the stack.
    /// To load the function, use `ldc` to load a function object.
    /// Arity is the operand.
    /// *2 bytes (1 operand)*
    Calli = 14,
    /// Creates a closure with a constant function and pushes it onto the stack.
    /// *Variable number of operands*
    Closure = 19,
}

/// Represents a chunk of bytecode.
#[derive(Debug, Clone)]
pub struct Chunk {
    /// A [`Vec`] of [`OpCode`]s and operands.
    pub code: Vec<u8>, // a byte array
    /// Source code positions for each byte in `code`.
    pub lines: Vec<usize>,
    /// Constant table for this [`Chunk`].
    pub constants: ValueArray,
    /// The name of the chunk.
    /// For most cases, should be the name of the function.
    /// If the [`Chunk`] is the top-level chunk, the name should `<global>`.
    pub name: String,
}

/// `u8` and `OpCode` should implement this trait.
pub trait ToByteCode {
    /// Transforms `self` into an `u8`.
    fn to_byte_code(&self) -> u8;
}

impl ToByteCode for OpCode {
    fn to_byte_code(&self) -> u8 {
        *self as u8
    }
}

impl ToByteCode for u8 {
    fn to_byte_code(&self) -> u8 {
        *self
    }
}

impl Chunk {
    /// Create an empty chunk with the specified `name`.
    /// 
    /// # Example
    /// ```
    /// use ella_value::chunk::Chunk;
    /// let chunk = Chunk::new("my_chunk".to_string());
    /// assert_eq!(chunk.name, "my_chunk");
    /// ```
    pub fn new(name: String) -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(),
            name,
        }
    }

    /// Write data to the [`Chunk`]. This can be an [`OpCode`] or an operand (`u8`).
    /// 
    /// # Params
    /// * `opcode` - The data to write to the chunk.
    /// * `line` - The original source line. This is used for runtime error messages and debugging.
    /// 
    /// # Example
    /// ```
    /// use ella_value::chunk::{Chunk, OpCode};
    /// let mut chunk = Chunk::new("my_chunk".to_string());
    /// chunk.write_chunk(OpCode::Ldc, 0);
    /// chunk.write_chunk(1, 0);
    /// assert_eq!(chunk.code, vec![0, 1]);
    /// assert_eq!(chunk.lines, vec![0, 0]);
    /// ```
    pub fn write_chunk(&mut self, opcode: impl ToByteCode, line: usize) {
        debug_assert_eq!(self.code.len(), self.lines.len());
        self.code.push(opcode.to_byte_code());
        self.lines.push(line);
        debug_assert_eq!(self.code.len(), self.lines.len());
    }

    /// Add a constant to the constant table.
    /// Returns the index of the added constant.
    /// 
    /// # Example
    /// ```
    /// use ella_value::chunk::Chunk;
    /// use ella_value::Value;
    /// let mut chunk = Chunk::new("my_chunk".to_string());
    /// let index = chunk.add_constant(Value::Bool(true));
    /// assert_eq!(index, 0);
    /// assert_eq!(chunk.constants, vec![Value::Bool(true)]);
    /// let index = chunk.add_constant(Value::Number(2.0));
    /// assert_eq!(index, 1);
    /// assert_eq!(chunk.constants, vec![Value::Bool(true), Value::Number(2.0)]);
    /// ```
    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        let loc = self.constants.len() - 1;
        if loc as u8 as usize != loc {
            todo!("load constant wide");
        }
        loc as u8
    }
}
