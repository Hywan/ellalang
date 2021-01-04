use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq)]
pub enum ObjKind {
    Str(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Obj {
    pub(crate) kind: ObjKind,
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
        }
    }
}
