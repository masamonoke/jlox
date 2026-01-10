use crate::{
    error::Error, expression::Expression, statement::Statement, token::{Literal, Token, TokenType}
};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    error: Error,
}

#[derive(Debug)]
pub struct ParseError;

// Grammar:
//
// program        → statement* EOF ;
// statement      → exprStmt
//                | printStmt ;
// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;
// expression     → equality ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" ;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            error: Error::new(),
        }
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_token(&[TokenType::Print]) {
            return self.print_statement()
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value")?;
        Ok(Statement::Print(expr))
    }

    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value")?;
        Ok(Statement::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expression, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.comparison();
        while self.match_token(&[TokenType::NotEqual, TokenType::EqualEqual]) {
            let op = self.previous();
            let right = self.comparison();
            if right.is_err() || expr.is_err() {
                todo!()
            }
            expr = Ok(Expression::Binary(
                Box::new(expr.unwrap()),
                op,
                Box::new(right.unwrap()),
            ));
        }
        expr
    }

    fn comparison(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.term();
        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous();
            let right = self.term();
            if expr.is_err() || right.is_err() {
                todo!()
            }
            expr = Ok(Expression::Binary(
                Box::new(expr.unwrap()),
                op,
                Box::new(right.unwrap()),
            ));
        }

        expr
    }

    fn term(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.factor();
        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let op = self.previous();
            let right = self.factor();
            if expr.is_err() {
                return Err(expr.err().unwrap());
            }
            if right.is_err() {
                return Err(right.err().unwrap());
            }
            expr = Ok(Expression::Binary(
                Box::new(expr.unwrap()),
                op,
                Box::new(right.unwrap()),
            ));
        }

        expr
    }

    fn factor(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.unary();
        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let op = self.previous();
            let right = self.unary();
            if right.is_err() || expr.is_err() {
                todo!()
            }
            expr = Ok(Expression::Binary(
                Box::new(expr.unwrap()),
                op,
                Box::new(right.unwrap()),
            ));
        }

        expr
    }

    fn unary(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(&[TokenType::Not, TokenType::Minus]) {
            let op = self.previous();
            let right = self.unary();
            if right.is_err() {
                return Err(self.report_error(self.peek().clone(), "Failed to match rhs value"))
            }
            return Ok(Expression::Unary(op, Box::new(right.unwrap())));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(&[TokenType::False]) {
            let literal = Literal::Bool(false);
            return Ok(Expression::Literal(literal));
        }
        if self.match_token(&[TokenType::True]) {
            let literal = Literal::Bool(true);
            return Ok(Expression::Literal(literal));
        }
        if self.match_token(&[TokenType::Nil]) {
            let literal = Literal::Nil;
            return Ok(Expression::Literal(literal));
        }

        if self.match_token(&[TokenType::Number, TokenType::String]) {
            return Ok(Expression::Literal(
                self.previous().literal.clone().unwrap(),
            ));
        }

        if self.match_token(&[TokenType::LeftParenthesis]) {
            let expr = self.expression();
            if expr.is_err() {
                todo!()
            }
            let _ = self.consume(TokenType::RightParenthesis, "Expect ')' after expression");
            return Ok(Expression::Grouping(Box::new(expr.unwrap())));
        }

        Err(self.report_error(self.peek().clone(), "Expected expression"))
    }

    fn match_token(&mut self, token_types: &[TokenType]) -> bool {
        let matched = token_types.iter().any(|token| self.check_token(token));
        if matched {
            self.advance();
        }
        matched
    }

    fn check_token(&self, typ: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().typ == *typ
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().typ == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn consume(&mut self, typ: TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check_token(&typ) {
            return Ok(self.advance());
        }

        Err(self.report_error(self.peek().clone(), message))
    }

    fn report_error(&mut self, token: Token, message: &str) -> ParseError {
        self.error.error(token, message);

        ParseError
    }

    // fn sync(&mut self) {
    //     self.advance();
    //
    //     while !self.is_at_end() {
    //         if self.previous().typ == TokenType::Semicolon {
    //             return;
    //         }
    //
    //         match self.peek().typ {
    //             TokenType::Class
    //             | TokenType::Fun
    //             | TokenType::Var
    //             | TokenType::For
    //             | TokenType::If
    //             | TokenType::While
    //             | TokenType::Print
    //             | TokenType::Return => return,
    //             _ => {}
    //         }
    //
    //         self.advance();
    //     }
    // }

    pub fn parse(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = vec![];

        while !self.is_at_end() {
            statements.push(self.statement()?);
        }

        Ok(statements)
    }

    // TODO: may be use it with cli arg passed
    // fn print_ast(expr: &Expression) -> String {
    //     let mut string = String::new();
    //     string += "(";
    //
    //     match expr {
    //         Expression::Binary(lhs, op, rhs) => {
    //             string += &(op.lexeme.clone() + " ");
    //             string += &(Self::print_ast(lhs) + " ");
    //             string += &Self::print_ast(rhs);
    //         },
    //         Expression::Grouping(group) => {
    //             string += "grouping ";
    //             string += &Self::print_ast(group);
    //         }
    //         Expression::Literal(liter) => {
    //             match liter {
    //                 Literal::Number(n) => return n.to_string(),
    //                 Literal::String(s) => return String::from(s),
    //                 Literal::Bool(b) => return b.to_string(),
    //                 Literal::Nil => return String::from("nil")
    //             }
    //         }
    //         Expression::Unary(lexeme, rhs) => {
    //             string += &(lexeme.lexeme.clone() + " ");
    //             string += &Self::print_ast(rhs);
    //         }
    //     }
    //
    //     string += ")";
    //     string
    // }

}
