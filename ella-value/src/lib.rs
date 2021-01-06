pub mod chunk;
pub mod object;
pub mod disassemble;

use std::fmt;
use std::rc::Rc;

#[derive(Clone, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Object(Rc<object::Obj>),
}

impl Value {
    /// Attempts to cast the `Value` into a `&str` or `None` if wrong type.
    pub fn cast_to_str(&self) -> Option<&str> {
        match self {
            Self::Object(obj) => match &obj.kind {
                object::ObjKind::Str(string) => Some(&string),
                #[allow(unreachable_patterns)] // when new object types are added
                _ => None,
            },
            _ => None,
        }
    }

    pub fn cast_to_number(&self) -> Option<f64> {
        match self {
            Self::Number(val) => Some(*val),
            _ => None,
        }
    }

    fn print_obj(f: &mut fmt::Formatter<'_>, obj: &object::Obj) -> fmt::Result {
        use object::ObjKind;
        match &obj.kind {
            ObjKind::Str(str) => write!(f, "{}", str),
            ObjKind::Fn { ident, .. } => write!(f, "<fn {}>", ident),
            ObjKind::NativeFn(object::NativeFn { ident, .. }) => write!(f, "<native fn {}>", ident),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(val) => write!(f, "{}", val),
            Value::Bool(val) => write!(f, "{}", val),
            Value::Object(val) => Self::print_obj(f, val),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

pub type ValueArray = Vec<Value>;
