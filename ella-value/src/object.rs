use crate::chunk::Chunk;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;

use super::Value;

#[derive(Clone)]
pub struct NativeFn {
    pub ident: String,
    pub arity: u32,
    pub func: &'static dyn Fn(&mut [Value]) -> Value,
}

#[derive(Clone)]
pub struct Function {
    pub ident: String,
    pub arity: u32,
    pub chunk: Chunk,
    pub upvalues_count: usize,
}

#[derive(Clone)]
pub struct Closure {
    pub func: Function,
    pub upvalues: Vec<Rc<RefCell<UpValue>>>,
}

#[derive(Clone)]
pub enum UpValue {
    Open(usize),
    Closed(Value),
}

impl UpValue {
    /// Returns `true` if the `UpValue` is in an open state ([`UpValue::Open`]).
    /// # Example
    /// ```
    /// use ella_value::object::UpValue;
    /// use ella_value::Value;
    /// 
    /// let upvalue = UpValue::Open(10);
    /// assert!(upvalue.is_open());
    /// let upvalue = UpValue::Closed(Value::Bool(false));
    /// assert!(!upvalue.is_open());
    /// ```
    pub fn is_open(&self) -> bool {
        matches!(self, UpValue::Open(_))
    }

    /// Returns `true` if the `UpValue` is in an open state ([`UpValue::Open`]) and has the given `index`.
    pub fn is_open_with_index(&self, index: usize) -> bool {
        matches!(self, UpValue::Open(x) if *x == index)
    }
}

#[derive(Clone)]
pub enum ObjKind {
    Str(String),
    Fn(Function),
    Closure(Closure),
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

const LOG_OBJECT_DROP: bool = false;

/// `Drop` is implemented for `Obj` merely to ease gc debugging.
impl Drop for Obj {
    fn drop(&mut self) {
        if LOG_OBJECT_DROP {
            match &self.kind {
                ObjKind::Str(string) => eprintln!("Collecting object {:?}", string),
                ObjKind::Fn(Function { ident, .. }) => {
                    eprintln!("Collecting function object {:?}", ident)
                }
                ObjKind::Closure(Closure { func, .. }) => {
                    eprintln!("Collecting closure object {:?}", func.ident)
                }
                ObjKind::NativeFn(NativeFn { ident, .. }) => {
                    eprintln!("Collecting native function object {:?}", ident)
                }
            }
        }
    }
}
