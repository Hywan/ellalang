use crate::value::{Value, ValueArray};
use enum_primitive_derive::Primitive;

/// Represents an opcode. Should only takes up a byte (`u8`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Primitive)]
#[repr(u8)]
pub enum OpCode {
    /// Load a constant onto the stack.
    /// *1 operand (2 bytes)*
    Ldc = 0,
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
}

pub struct Chunk {
    pub(crate) code: Vec<u8>, // a byte array
    /// Source code positions for each byte in `code`.
    pub(crate) lines: Vec<usize>,
    pub(crate) constants: ValueArray,
}

/// `u8` and `OpCode` should implement this trait.
pub trait ToByteCode {
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
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(),
        }
    }

    /// Write data to the `Chunk`.
    pub fn write_chunk(&mut self, opcode: impl ToByteCode, line: usize) {
        debug_assert_eq!(self.code.len(), self.lines.len());
        self.code.push(opcode.to_byte_code());
        self.lines.push(line);
        debug_assert_eq!(self.code.len(), self.lines.len());
    }

    /// Add a constant to the constant table.
    /// Returns the index of the added constant.
    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        let loc = self.constants.len() - 1;
        if loc as u8 as usize != loc {
            todo!("load constant wide");
        }
        loc as u8
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
