use crate::chunk::Chunk;
use std::cmp::Ordering;

use super::Value;

#[derive(Clone)]
pub struct NativeFn {
    pub ident: String,
    pub arity: u32,
    pub func: &'static dyn Fn(&mut [Value]) -> Value,
}

#[derive(Clone)]
pub enum ObjKind {
    Str(String),
    Fn {
        ident: String,
        /// Number of arguments that the function accepts.
        arity: u32,
        chunk: Chunk,
    },
    NativeFn(NativeFn),
}

impl PartialEq for ObjKind {
    fn eq(&self, other: &ObjKind) -> bool {
        match self {
            Self::Str(l) => match other {
                Self::Str(r) => l == r,
                _ => false,
            },
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct Obj {
    pub kind: ObjKind,
}

impl Obj {
    pub fn new_string(str: String) -> Self {
        Self {
            kind: ObjKind::Str(str),
        }
    }
}

impl PartialOrd for Obj {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        None // object ordering is always false
    }
}

/// `Drop` is implemented for `Obj` merely to ease gc debugging.
impl Drop for Obj {
    fn drop(&mut self) {
        match &self.kind {
            ObjKind::Str(string) => eprintln!("Collecting object {:?}", string),
            ObjKind::Fn { ident, .. } => eprintln!("Collecting function object {:?}", ident),
            ObjKind::NativeFn(NativeFn { ident, .. }) => {
                eprintln!("Collecting native function object {:?}", ident)
            }
        }
    }
}
