use std::{cell::RefCell, ops::Range};

pub struct Source<'a> {
    pub content: &'a str,
    pub errors: ErrorReporter,
}

impl<'a> Source<'a> {
    pub fn new(content: &'a str) -> Self {
        Self {
            content,
            errors: ErrorReporter::new(),
        }
    }
}

impl<'a> Into<Source<'a>> for &'a str {
    fn into(self) -> Source<'a> {
        Source::new(self)
    }
}

#[derive(Debug, Clone)]
pub struct SyntaxError {
    message: String,
    span: Range<usize>,
}

impl SyntaxError {
    pub fn new(message: impl ToString, span: Range<usize>) -> Self {
        Self {
            message: message.to_string(),
            span,
        }
    }
}

/// Manages all the errors
pub struct ErrorReporter {
    errors: RefCell<Vec<SyntaxError>>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            errors: RefCell::new(Vec::new()),
        }
    }

    pub fn add_error(&self, error: SyntaxError) {
        self.errors.borrow_mut().push(error); // this should be the only place where self.errors is borrowed mutably
    }
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self::new()
    }
}
