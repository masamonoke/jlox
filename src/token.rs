#[derive(Clone, Debug, PartialEq)]
pub enum TokenType {
    LeftParenthesis,
    RightParenthesis,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Not,
    NotEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Identifier,
    String,
    Number,
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof
}

#[derive(Clone, Debug)]
pub enum Literal {
    Number(f32),
    String(String),
    Bool(bool),
    Nil
}

#[derive(Clone, Debug)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: u32
}
