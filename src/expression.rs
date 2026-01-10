use crate::token::{Token, Literal};

#[derive(Debug)]
pub enum Expression {
    Binary(Box<Expression>, Token, Box<Expression>),
    Unary(Token, Box<Expression>),
    Literal(Literal),
    Grouping(Box<Expression>),
}
