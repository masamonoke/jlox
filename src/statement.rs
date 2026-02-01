use crate::{expression::Expression, token::Token};

pub enum Statement {
    Expression(Expression),
    Print(Expression),
    Variable(Token, Option<Expression>)
}
