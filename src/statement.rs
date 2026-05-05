use crate::{expression::Expression, token::Token};

pub enum Statement {
    Expression(Expression),
    Print(Expression),
    Variable(Token, Option<Expression>),
    Block(Vec<Statement>),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
    While(Expression, Box<Statement>)
}
