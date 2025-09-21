use crate::token::{Token, TokenType};

pub struct Error {
    pub had_error: bool
}

impl Error {
    pub fn new() -> Error {
        Error { had_error: false }
    }

    pub fn error(&mut self, token: Token, message: &str) {
        self.had_error = true;
        if token.typ == TokenType::Eof {
            self.report_eof(token.line, message)
        } else {
            self.report_error(token.line, "", message)
        }
    }

    fn report_error(&mut self, line: u32, location: &str, message: &str) {
        eprintln!("[line {}] Error {}: {}", line, location, message);
    }

    fn report_eof(&mut self, line: u32, message: &str) {
        eprintln!("[line {}] at end: {}", line, message);
    }
}

impl Default for Error {
    fn default() -> Self {
        Error::new()
    }
}
