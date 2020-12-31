use crate::chunk::{Chunk, OpCode};
use num_traits::FromPrimitive;
use std::fmt;

impl Chunk {
    /// Disassemble simple (1 byte) instruction.
    fn simple_instr(
        &self,
        f: &mut fmt::Formatter<'_>,
        name: &str,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        writeln!(f, "{}", name)?;
        Ok(offset + 1)
    }

    /// Disassemble ldc (2 bytes) instruction.
    fn constant_instr(
        &self,
        f: &mut fmt::Formatter<'_>,
        name: &str,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        let constant = self.constants[self.code[offset + 1] as usize];
        writeln!(f, "{:<5} {}", name, constant)?;
        Ok(offset + 2)
    }

    /// Disassembles the instruction at the `offset`.
    fn disassemble_instr(
        &self,
        f: &mut fmt::Formatter<'_>,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        write!(f, "{:04} ", offset)?;

        let instr = self.code[offset];

        // print source line number
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            write!(f, "{:>4} ", "|")?;
        } else {
            write!(f, "{:>4} ", self.lines[offset])?;
        }

        // SAFETY:
        // If not a valid OpCode, none of the branches should match and thus cause an error.
        match OpCode::from_u8(instr) {
            Some(OpCode::Ldc) => self.constant_instr(f, "ldc", offset),
            Some(OpCode::Neg) => self.simple_instr(f, "neg", offset),
            Some(OpCode::Not) => self.simple_instr(f, "not", offset),
            Some(OpCode::Add) => self.simple_instr(f, "add", offset),
            Some(OpCode::Sub) => self.simple_instr(f, "sub", offset),
            Some(OpCode::Mul) => self.simple_instr(f, "mul", offset),
            Some(OpCode::Div) => self.simple_instr(f, "div", offset),
            Some(OpCode::Ret) => self.simple_instr(f, "ret", offset),
            Some(OpCode::LdTrue) => self.simple_instr(f, "ld_true", offset),
            Some(OpCode::LdFalse) => self.simple_instr(f, "ld_false", offset),
            None => self.simple_instr(f, "invalid", offset), // skip bad instruction
        } // returns the next ip
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "== {} ==", "chunk")?;

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instr(f, offset)?;
        }

        Ok(())
    }
}
