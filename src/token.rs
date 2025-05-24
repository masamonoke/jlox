#[derive(Clone, Debug)]
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

#[derive(Clone)]
pub struct Literal {
    num: Option<f32>,
    str: Option<String>
}

impl Literal {
    pub fn new_integral(num: f32) -> Literal {
        Literal { num: Some(num), str: None }
    }

    pub fn new_string(str: String) -> Literal {
        Literal { num: None, str: Some(str) }
    }

    pub fn num(&self) -> &Option<f32> {
        return &self.num;
    }

    pub fn string(&self) -> Option<&String> {
        return self.str.as_ref();
    }
}

pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: u32
}
