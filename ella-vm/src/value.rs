pub mod object;

use std::fmt;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Object(Box<object::Obj>),
}

impl Value {
    fn print_obj(f: &mut fmt::Formatter<'_>, obj: &object::Obj) -> fmt::Result {
        use object::ObjKind;
        match &obj.kind {
            ObjKind::Str(str) => write!(f, "{}", str),
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

pub type ValueArray = Vec<Value>;
