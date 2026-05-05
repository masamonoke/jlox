use std::rc::Rc;

use crate::{
    environment::{self, Environment},
    expression::Expression,
    statement::Statement,
    token::{Literal, Token, TokenType},
    value::{Number, Value},
};
use anyhow::{anyhow, Result};

pub struct Interpreter {
    env: Rc<environment::Environment>,
}

// TODO: add more info to errors like line where it happend and so on
impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            env: Rc::new(Environment::new()),
        }
    }

    pub fn interpret_statements(&mut self, statements: Vec<Statement>) -> Result<(), anyhow::Error> {
        for stmt in statements {
            self.execute(&stmt)?;
        }

        Ok(())
    }

    fn execute(&mut self, statement: &Statement) -> Result<(), anyhow::Error> {
        match statement {
            Statement::Expression(expr) => {
                self.evaluate(expr)?; // TODO: handle value?
            }
            Statement::Print(expr) => {
                let value = self.evaluate(expr)?;
                println!("{}", stringify(value)?);
            }
            Statement::Variable(token, initializer) => {
                let mut value: Option<Value> = None;
                if initializer.is_some() {
                    value = Some(self.evaluate(initializer.as_ref().unwrap())?);
                }

                self.env.define(token.lexeme.clone(), value);
            }
            Statement::Block(list) => {
                self.execute_block(list, Environment::from(self.env.clone()))?;
            },
            Statement::If(cond, then_scope, else_scope) => {
                if let Value::Bool(cond_res) = self.evaluate(cond)? {
                    if cond_res {
                        let _ = self.execute(then_scope.as_ref());
                    } else if else_scope.is_some() {
                        let _ = self.execute(else_scope.as_ref().unwrap().as_ref());
                    }
                }
            }
        }

        Ok(())
    }

    fn execute_block(&mut self, stmts: &Vec<Statement>, env: Environment) -> Result<(), anyhow::Error>{
        let prev_env = self.env.clone();
        self.env = Rc::new(env);
        for stmt in stmts {
            self.execute(stmt)?
        }
        self.env = prev_env;
        Ok(())
    }

    fn evaluate(&self, expr: &Expression) -> Result<Value> {
        match expr {
            Expression::Binary(lhs, op, rhs) => self.binary(lhs, rhs, &op.typ),
            Expression::Unary(lexeme, rhs) => self.unary(lexeme, rhs),
            Expression::Grouping(group) => self.evaluate(group),
            Expression::Literal(lit) => literal(lit),
            Expression::Variable(token) => {
                if !self.env.contains(&token.lexeme) {
                    return Err(anyhow!("Undefined variable '{}", &token.lexeme));
                }

                let value = self.env.get(&token.lexeme);
                if let Some(value) = value {
                    return Ok(value)
                }
                Err(anyhow!("Usage of uninitialized variable '{}'", &token.lexeme))
            },
            Expression::Assign(tok, expr) => {
                if !self.env.contains(&tok.lexeme) {
                    return Err(anyhow!("{} is not declared", &tok.lexeme))
                }

                let rhs = self.evaluate(expr);
                if let Ok(rhs) = rhs {
                    self.env.define(tok.lexeme.clone(), Some(rhs.clone()));
                    return Ok(rhs)
                }

                Err(anyhow!("{}", rhs.err().unwrap()))
            },
            Expression::Logical(lhs_ptr, op, rhs_ptr) => {
                let lhs = self.evaluate(lhs_ptr)?;
                let is_left = is_truthy(&lhs);
                return match op.typ {
                    TokenType::Or if is_left => Ok(lhs),
                    TokenType::And if !is_left => Ok(lhs),
                    _ => self.evaluate(rhs_ptr)
                }
            }
        }
    }

    fn binary(&self, lhs: &Expression, rhs: &Expression, typ: &TokenType) -> Result<Value> {
        let left = self.evaluate(lhs);
        let right = self.evaluate(rhs);

        if let Some(left_err) = left.as_ref().err() {
            return Err(anyhow!("{}", left_err))
        }

        if let Some(right_err) = right.as_ref().err() {
            return Err(anyhow!("{}", right_err))
        }

        let left = left.unwrap();
        let right = right.unwrap();

        let handle_num = |f: fn(Number, Number) -> Number, error: anyhow::Error| -> Result<Value> {
            if let Value::Number(left) = left {
                if let Value::Number(right) = right {
                    return Ok(Value::Number(f(left, right)));
                }
            }

            Err(error)
        };

        let handle_bool = |f: fn(Number, Number) -> bool| -> Result<Value> {
            if let Value::Number(left) = left {
                if let Value::Number(right) = right {
                    return Ok(Value::Bool(f(left, right)));
                }
            }

            Err(anyhow!("Undefined op"))
        };

        match typ {
            TokenType::Minus => {
                handle_num(|left, right| left - right, anyhow!("Failed to map minus"))
            }
            TokenType::Slash => handle_num(
                |left, right| left / right,
                anyhow!("Failed to map division"),
            ),
            TokenType::Star => handle_num(
                |left, right| left * right,
                anyhow!("Failed to map multiply"),
            ),
            TokenType::Plus => {
                if let Value::Number(_) = left {
                    return handle_num(|left, right| left + right, anyhow!("Failed to map plus"));
                }

                if let Value::String(left) = left {
                    if let Value::String(right) = right {
                        return Ok(Value::String(left + &right));
                    }
                }

                Err(anyhow!("Failed to map plus operator"))
            }
            TokenType::Greater => handle_bool(|left, right| left > right),
            TokenType::GreaterEqual => handle_bool(|left, right| left >= right),
            TokenType::Less => handle_bool(|left, right| left < right),
            TokenType::LessEqual => handle_bool(|left, right| left <= right),
            TokenType::NotEqual => handle_bool(|left, right| left != right),
            TokenType::EqualEqual => handle_bool(|left, right| left == right),
            _ => todo!(),
        }
    }

    fn unary(&self, lexeme: &Token, rhs: &Expression) -> Result<Value> {
        let right = self.evaluate(rhs);

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
            }
            TokenType::Not => {
                if let Value::Bool(right) = right {
                    return Ok(Value::Bool(!right));
                }

                Err(anyhow!("Not a boolean"))
            }
            _ => todo!(),
        }
    }

}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

fn stringify(obj: Value) -> Result<String> {
    match obj {
        Value::String(s) => Ok(s),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Nil => Ok("nil".to_string()),
        Value::Number(n) => Ok(n.to_string()),
    }
}

fn literal(lit: &Literal) -> Result<Value> {
    match lit {
        Literal::Number(n) => Ok(Value::Number(*n)),
        Literal::String(s) => Ok(Value::String(s.clone())),
        Literal::Bool(b) => Ok(Value::Bool(*b)),
        Literal::Nil => Ok(Value::Nil),
    }
}

fn is_truthy(val: &Value) -> bool {
    match val {
        Value::Bool(b) => *b == true,
        // TODO: probably need to compare delta with epsilon or use separate type for floats
        Value::Number(n) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Nil => false,
    }
}
