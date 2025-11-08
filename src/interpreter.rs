use crate::{parser::Expression, token::{Literal, Token, TokenType}};
use anyhow::{anyhow, Ok, Result};

type Number = f32;

enum Value {
    Bool(bool),
    Number(Number),
    String(String),
    Nil
}

pub fn interpret(expr: Expression) {
    let _= evaluate(expr)
        .and_then(stringify)
        .map(|output| println!("{}", output));
}

fn evaluate(expr: Expression) -> Result<Value> {
    match expr {
        Expression::Binary(lhs, op, rhs) => {
            binary(*lhs, *rhs, &op.typ)
        },
        Expression::Unary(lexeme, rhs) => {
            unary(&lexeme, *rhs)
        },
        Expression::Grouping(group) => {
            evaluate(*group)
        },
        Expression::Literal(lit) => {
            literal(lit)
        }
    }
}

fn stringify(obj: Value) -> Result<String> {
    match obj {
        Value::String(s) => Ok(s),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Nil => Ok("nil".to_string()),
        Value::Number(n) => Ok(n.to_string())
    }
}

fn binary(lhs: Expression, rhs: Expression, typ: &TokenType) -> Result<Value> {
    let left = evaluate(lhs);
    let right = evaluate(rhs);

    if left.is_err() || right.is_err() {
        return Err(anyhow!("error"));
    }

    let left = left.unwrap();
    let right = right.unwrap();

    let handle_num = |f: fn(Number, Number) -> Number, error: anyhow::Error| -> Result<Value> {
        if let Value::Number(left) = left {
            if let Value::Number(right) = right {
                return Ok(Value::Number(f(left, right)))
            }
        }

        Err(error)
    };

    let handle_bool = |f: fn(Number, Number) -> bool| -> Result<Value> {
        if let Value::Number(left) = left {
            if let Value::Number(right) = right {
                return Ok(Value::Bool(f(left, right)))
            }
        }

        Err(anyhow!("Undefined op"))
    };

    match typ {
        TokenType::Minus => {
            handle_num(|left, right| left - right, anyhow!("Failed to map minus"))
        }
        TokenType::Slash => {
            handle_num(|left, right| left / right, anyhow!("Failed to map division"))
        }
        TokenType::Star => {
            handle_num(|left, right| left * right, anyhow!("Failed to map multiply"))
        }
        TokenType::Plus => {
            if let Value::Number(_) = left {
                return handle_num(|left, right| left + right, anyhow!("Failed to map plus"))
            }

            if let Value::String(left) = left {
                if let Value::String(right) = right {
                    return Ok(Value::String(left + &right));
                }
            }

            Err(anyhow!("Failed to map plus operator"))
        },
        TokenType::Greater => handle_bool(|left, right| left > right),
        TokenType::GreaterEqual => handle_bool(|left, right| left >= right),
        TokenType::Less => handle_bool(|left, right| left < right),
        TokenType::LessEqual => handle_bool(|left, right| left <= right),
        TokenType::NotEqual => handle_bool(|left, right| left != right),
        TokenType::EqualEqual => handle_bool(|left, right| left == right),
        _ => todo!(),
    }
}

fn unary(lexeme: &Token, rhs: Expression) -> Result<Value> {
    let right = evaluate(rhs);

    if right.is_err() {
        return Err(anyhow!("error"));
    }

    let right = right.unwrap();

    match lexeme.typ {
        TokenType::Minus => {
            if let Value::Number(right) = right {
                return Ok(Value::Number(-right));
            }

            Err(anyhow!("Not a number"))
        },
        TokenType::Not => {
            if let Value::Bool(right) = right {
                return Ok(Value::Bool(!right))
            }

            Err(anyhow!("Not a boolean"))
        },
        _ => todo!()
    }
}

fn literal(lit: Literal) -> Result<Value> {
    match lit {
        Literal::Number(n) => Ok(Value::Number(n)),
        Literal::String(s) => Ok(Value::String(s)),
        Literal::Bool(b) => Ok(Value::Bool(b)),
        Literal::Nil => Ok(Value::Nil)
    }
}
