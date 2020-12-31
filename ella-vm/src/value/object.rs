use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum ObjKind {
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Obj {
    pub(crate) kind: ObjKind,
    pub(crate) next: *mut Obj,
}

impl Obj {
    pub fn new_string(str: String) -> Self {
        Self {
            kind: ObjKind::Str(str),
            next: std::ptr::null_mut(),
        }
    }
}

impl PartialOrd for Obj {
    fn partial_cmp(&self, _other: &Self) -> Option<Ordering> {
        None // object ordering is always false
    }
}
