use std::collections::HashMap;

use crate::token::Literal;
use crate::token::TokenType;
use crate::token::Token;


pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: u32,
    keywords: HashMap<String, TokenType>
}

impl Scanner {
    pub fn new(source: Vec<char>) -> Scanner {
        let keywords: HashMap<String, TokenType> = [
            ("and".to_string(), TokenType::And),
            ("class".to_string(), TokenType::Class),
            ("else".to_string(), TokenType::Else),
            ("false".to_string(), TokenType::False),
            ("for".to_string(), TokenType::For),
            ("fun".to_string(), TokenType::Fun),
            ("if".to_string(), TokenType::If),
            ("nil".to_string(), TokenType::Nil),
            ("or".to_string(), TokenType::Or),
            ("print".to_string(), TokenType::Print),
            ("return".to_string(), TokenType::Print),
            ("super".to_string(), TokenType::Super),
            ("this".to_string(), TokenType::This),
            ("true".to_string(), TokenType::True),
            ("var".to_string(), TokenType::Var),
            ("while".to_string(), TokenType::While)
        ].into();
        Scanner { source, tokens: vec![], start: 0, current: 0, line: 1, keywords }
    }

    pub fn scan(&mut self) {
        while !self.is_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token { typ: TokenType::Eof, lexeme: "".to_string(), literal: None, line: self.line });
    }

    fn is_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    fn scan_token(&mut self) {
        let symbol = self.advance();
        if !self.match_symbol(symbol) {
            eprintln!("Unexpected symbol: {}", symbol)
        }
    }

    fn match_symbol(&mut self, symbol: char) -> bool {
        self.match_brace(symbol) ||
        self.match_escape(symbol) ||
        self.match_string() ||
        self.match_operator(symbol) ||
        self.match_number(symbol) ||
        self.match_identifier(symbol)
    }

    fn match_operator(&mut self, symbol: char) -> bool {
        match symbol {
            ',' => self.add_token_without_lexeme(TokenType::Comma),
            '.' => self.add_token_without_lexeme(TokenType::Dot),
            '-' => self.add_token_without_lexeme(TokenType::Minus),
            '+' => self.add_token_without_lexeme(TokenType::Plus),
            ';' => self.add_token_without_lexeme(TokenType::Semicolon),
            '*' => self.add_token_without_lexeme(TokenType::Star),
            '!' => self.add_long_operator('=', TokenType::NotEqual, TokenType::Equal),
            '=' => self.add_long_operator('=', TokenType::EqualEqual, TokenType::Equal),
            '<' => self.add_long_operator('=', TokenType::LessEqual, TokenType::Less),
            '>' => self.add_long_operator('>', TokenType::GreaterEqual, TokenType::Greater),
            '/' => {
                if self.match_and_advance('/') {
                    while self.peek().is_some_and(|c| c != '\n') && !self.is_end() {
                        self.advance();
                    }
                } else {
                    self.add_token_without_lexeme(TokenType::Slash);
                }
            }
            _ => return false
        }

        true
    }

    fn match_brace(&mut self, symbol: char) -> bool {
        match symbol {
            '(' => self.add_token_without_lexeme(TokenType::LeftParenthesis),
            ')' => self.add_token_without_lexeme(TokenType::RightParenthesis),
            '{' => self.add_token_without_lexeme(TokenType::LeftBrace),
            '}' => self.add_token_without_lexeme(TokenType::RightBrace),
            _ => return false
        }

        true
    }

    fn match_escape(&mut self, symbol: char) -> bool {
        match symbol {
            ' ' | '\r' | '\t' => {},
            '\n' => self.line += 1,
            _ => return false
        }

        true
    }

    fn match_string(&mut self) -> bool {
        if self.source[self.current - 1] != '"' {
            return false;
        }

        while self.peek().is_some_and(|c| c != '"') && !self.is_end() {
            if self.peek().is_some_and(|c| c == '\n') {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_end() {
            // TODO: replace with custom error log
            eprintln!("Unterminated string found.");
            return false;
        }

        self.advance();
        let string: String = self.source[self.start + 1..self.current - 1].iter().collect();
        self.add_token(TokenType::String, Some(Literal::new_string(string)));

        return true
    }

    fn match_number(&mut self, symbol: char) -> bool {
        if !Scanner::is_digit(symbol) {
            return false
        }

        while self.peek().map_or(false, Scanner::is_digit) {
            self.advance();
        }

        let is_next_digit = self.peek_next().is_some_and(|c| Scanner::is_digit(c));
        if self.peek().is_some_and(|c| c == '.') && is_next_digit {
            self.advance();
            while self.peek().map_or(false, Scanner::is_digit) {
                self.advance();
            }
        }

        let number: String = self.source[self.start..self.current].iter().collect();
        let number: f32 = number.parse().unwrap();
        self.add_token(TokenType::Number, Some(Literal::new_integral(number)));

        return true
    }

    fn match_identifier(&mut self, symbol: char) -> bool {
        if Scanner::is_digit(symbol) {
            return false;
        }

        let is_identifier = |symbol: char| {
            let is_alphabet = (symbol >= 'a' && symbol <= 'z') ||
            (symbol >= 'A' && symbol <= 'Z') || (symbol  == '_');
            return is_alphabet || Scanner::is_digit(symbol);
        };

        if !is_identifier(symbol) {
            return false;
        }

        while self.peek().is_some_and(|c| is_identifier(c)) {
            self.advance();
        }

        let indentifier_name: String = self.source[self.start..self.current].iter().collect();
        if self.keywords.contains_key(&indentifier_name) {
            self.add_token_without_lexeme(self.keywords[&indentifier_name].clone());
        } else {
            self.add_token_without_lexeme(TokenType::Identifier);
        }

        return true;
    }

    fn is_digit(symbol: char) -> bool {
        symbol >= '0' && symbol <= '9'
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        return c;
    }

    fn add_token_without_lexeme(&mut self, typ: TokenType) {
        self.add_token(typ, None);
    }

    fn add_long_operator(&mut self, expected_symbol: char, long_type: TokenType, short_type: TokenType) {
        if self.match_and_advance(expected_symbol) {
            self.add_token_without_lexeme(long_type);
        } else {
            self.add_token_without_lexeme(short_type);
        }
    }

    fn add_token(&mut self, typ: TokenType, literal: Option<Literal>) {
        let lexeme = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token { typ, lexeme, literal, line: self.line });
    }

    fn match_and_advance(&mut self, expected: char) -> bool {
        if self.is_end() {
            return false;
        }

        if self.source[self.current] != expected {
            return false;
        }

        self.current += 1;
        return true;
    }

    fn peek(&self) -> Option<char> {
        if self.is_end() {
            return None
        }

        Some(self.source[self.current])
    }

    fn peek_next(&self) -> Option<char> {
        if self.current + 1 > self.source.len() {
            return None
        }

        return Some(self.source[self.current + 1]);
    }

    pub fn log(&self) {
        self.tokens.iter().for_each(|t| {
            let token_type = t.typ.clone();
            let lexeme = t.lexeme.clone();
            if t.literal.is_some() {
                let literal = t.literal.clone().unwrap();
                if literal.num().is_some() {
                    println!("Token type: {:?}, lexeme: {}, literal: {}", token_type, lexeme, literal.num().unwrap());
                } else {
                    println!("Token type: {:?}, lexeme: {}, literal: {}", token_type, lexeme, literal.string().unwrap());
                }
            } else {
                println!("Token type: {:?}, lexeme: {}", token_type, lexeme);
            }
        });
    }
}
