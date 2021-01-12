use crate::chunk::{Chunk, OpCode};
use crate::object::ObjKind;
use crate::Value;
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
        let constant_index = self.code[offset + 1];
        let constant = self.constants[constant_index as usize].clone();
        writeln!(
            f,
            "{:<5} {:<3} (value = {})",
            name, constant_index, constant
        )?;
        Ok(offset + 2)
    }

    /// Disassemble ldloc, stloc, ldupval and stupval (2 bytes) instruction.
    fn ld_or_st_instr(
        &self,
        f: &mut fmt::Formatter<'_>,
        name: &str,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        let var_offset = self.code[offset + 1];
        writeln!(f, "{:<5} {}", name, var_offset)?;
        Ok(offset + 2)
    }

    /// Disassemble calli (2 bytes) instruction.
    fn calli_instr(
        &self,
        f: &mut fmt::Formatter<'_>,
        name: &str,
        offset: usize,
    ) -> Result<usize, fmt::Error> {
        let arity = self.code[offset + 1];
        writeln!(f, "{:<5} {}", name, arity)?;
        Ok(offset + 2)
    }

    fn closure_instr(
        &self,
        f: &mut fmt::Formatter<'_>,
        name: &str,
        mut offset: usize,
    ) -> Result<usize, fmt::Error> {
        let constant_index = self.code[offset + 1];
        let constant = self.constants[constant_index as usize].clone();
        writeln!(
            f,
            "{:<5} {:<3} (value = {})",
            name, constant_index, constant
        )?;
        offset += 2;

        if let Value::Object(obj) = constant {
            if let ObjKind::Fn(func) = &obj.kind {
                for _i in 0..func.upvalues_count {
                    let is_local = self.code[offset];
                    let index = self.code[offset + 1];
                    writeln!(
                        f,
                        "{:04} {:>4}  {:>7}{:>5}",
                        offset - 2,
                        "|",
                        if is_local != 0 { "local" } else { "upvalue" },
                        index
                    )?;
                    offset += 2;
                }
            } else {
                unreachable!();
            }
        } else {
            unreachable!()
        }

        Ok(offset)
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
            Some(OpCode::LdLoc) => self.ld_or_st_instr(f, "ldloc", offset),
            Some(OpCode::StLoc) => self.ld_or_st_instr(f, "stloc", offset),
            Some(OpCode::LdUpVal) => self.ld_or_st_instr(f, "ldupval", offset),
            Some(OpCode::StUpVal) => self.ld_or_st_instr(f, "stupval", offset),
            Some(OpCode::CloseUpVal) => self.simple_instr(f, "closeupval", offset),
            Some(OpCode::Neg) => self.simple_instr(f, "neg", offset),
            Some(OpCode::Not) => self.simple_instr(f, "not", offset),
            Some(OpCode::Add) => self.simple_instr(f, "add", offset),
            Some(OpCode::Sub) => self.simple_instr(f, "sub", offset),
            Some(OpCode::Mul) => self.simple_instr(f, "mul", offset),
            Some(OpCode::Div) => self.simple_instr(f, "div", offset),
            Some(OpCode::Ret) => self.simple_instr(f, "ret", offset),
            Some(OpCode::LdTrue) => self.simple_instr(f, "ld_true", offset),
            Some(OpCode::LdFalse) => self.simple_instr(f, "ld_false", offset),
            Some(OpCode::Eq) => self.simple_instr(f, "eq", offset),
            Some(OpCode::Greater) => self.simple_instr(f, "greater", offset),
            Some(OpCode::Less) => self.simple_instr(f, "less", offset),
            Some(OpCode::Pop) => self.simple_instr(f, "pop", offset),
            Some(OpCode::Calli) => self.calli_instr(f, "calli", offset),
            Some(OpCode::Closure) => self.closure_instr(f, "closure", offset),
            None => self.simple_instr(f, "invalid", offset), // skip bad instruction
        } // returns the next ip
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "== {} ==", self.name)?;

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instr(f, offset)?;
        }

        Ok(())
    }
}
