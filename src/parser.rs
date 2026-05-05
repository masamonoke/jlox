use crate::{
    error::Error,
    expression::Expression,
    statement::Statement,
    token::{Literal, Token, TokenType},
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
// program        → declaration* EOF ;
// declaration    → varDecl
//                | statement ;
// varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
// statement      → exprStmt
//                | printStmt
//                | block
//                | if_statement ;
// forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
//                  expression? ";"
//                  expression? ")" statement ;
// whileStmt      → "while" "(" expression ")" statement ;
// block          → "{" declaration "}" ;
// exprStmt       → expression ";" ;
// printStmt      → "print" expression ";" ;
// if_statement   → "if" "(" expression ")" statement
//                   ( "else" statement )? ;
// expression     → assignment ;
// assignment     → logic_or | IDENTIFIER "=" assignment ;
// logic_or       → logic_and ( "or" logic_and )* ;
// logic_and      → equality ( "and" equality )* ;
// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
// term           → factor ( ( "-" | "+" ) factor )* ;
// factor         → unary ( ( "/" | "*" ) unary )* ;
// unary          → ( "!" | "-" ) unary
//                | primary ;
// primary        → NUMBER | STRING | "true" | "false" | "nil"
//                | "(" expression ")" | IDENTIFIER;

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens,
            current: 0,
            error: Error::new(),
        }
    }

    // TODO: maybe return list or errors and the caller will output them if needed and etc?
    pub fn parse(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = vec![];

        while !self.is_at_end() {
            let decl = self.declaration();
            if decl.is_err() {
                self.sync();
                continue;
            }
            statements.push(decl.unwrap());
        }

        if self.error.had_error {
            return Err(ParseError);
        }

        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Statement, ParseError> {
        if self.match_token(&[TokenType::Var]) {
            return self.var_decl();
        }

        self.statement()
    }

    fn var_decl(&mut self) -> Result<Statement, ParseError> {
        let name = self.consume(TokenType::Identifier, "Expected variable name")?;
        let mut expr = None;
        if self.match_token(&[TokenType::Equal]) {
            expr = Some(self.expression()?);
        }
        let _ = self.consume(
            TokenType::Semicolon,
            "Expected ';' after variable declaration",
        );

        Ok(Statement::Variable(name, expr))
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        if self.match_token(&[TokenType::Print]) {
            return self.print_statement();
        }

        if self.match_token(&[TokenType::LeftBrace]) {
            return self.block();
        }

        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.match_token(&[TokenType::For]) {
            return self.for_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value")?;
        Ok(Statement::Print(expr))
    }

    fn block(&mut self) -> Result<Statement, ParseError> {
        let mut stmts = vec![];
        while !self.check_token(&TokenType::RightBrace) {
            // TODO: what if declaration is erroneous?
            stmts.push(self.declaration().unwrap());
        }

        let _ = self.consume(TokenType::RightBrace, "Expected '}' after block");
        Ok(Statement::Block(stmts))
    }

    fn if_statement(&mut self) -> Result<Statement, ParseError> {
        let _ = self.consume(TokenType::LeftParenthesis, "Expected '(' after 'if'");
        let cond = self.expression()?;
        let _ = self.consume(TokenType::RightParenthesis, "Expected ')' after 'if'");

        let then_branch = Box::new(self.statement()?);
        let mut else_branch = None;
        if self.match_token(&[TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()?))
        }

        Ok(Statement::If(cond, then_branch, else_branch))
    }

    fn while_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenType::LeftParenthesis, "Expected '(' after 'while'.")?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParenthesis, "Expected ')' after 'while'.")?;
        let body = self.statement()?;
        Ok(Statement::While(cond, Box::new(body)))
    }

    fn for_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenType::LeftParenthesis, "Expected '(' after 'for'.")?;
        let init = if self.match_token(&[TokenType::Semicolon]) {
            None
        } else if self.match_token(&[TokenType::Var]) {
            Some(self.var_decl()?)
        } else {
            Some(self.expression_statement()?)
        };

        let mut cond: Option<Expression> = None;
        if !self.check_token(&TokenType::Semicolon) {
            cond = Some(self.expression()?);
        }
        self.consume(TokenType::Semicolon, "Expected ';' after loop condition.")?;

        let mut inc: Option<Expression> = None;
        if !self.check_token(&TokenType::RightParenthesis) {
            inc = Some(self.expression()?);
        }
        self.consume(
            TokenType::RightParenthesis,
            "Expected ')' after for clauses.",
        )?;

        let mut body = self.statement()?;
        if let Some(inc) = inc {
            body = Statement::Block(vec![body, Statement::Expression(inc)]);
        }

        if cond.is_none() {
            cond = Some(Expression::Literal(Literal::Bool(true)));
        }

        body = Statement::While(cond.unwrap(), Box::new(body));

        if let Some(init) = init {
            body = Statement::Block(vec![init, body]);
        }

        Ok(body)
    }

    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after value")?;
        Ok(Statement::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expression, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expression, ParseError> {
        let expr = self.logic_or();
        if !self.match_token(&[TokenType::Equal]) {
            return expr;
        }

        let lhs = expr;
        if let Expression::Variable(tok) = lhs? {
            let rhs = self.assignment()?;
            return Ok(Expression::Assign(tok, Box::new(rhs)));
        }

        Err(self.report_error(self.peek().clone(), "Failed to match assignment"))
    }

    fn logic_or(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.logic_and()?;
        while self.match_token(&[TokenType::Or]) {
            let op = self.previous();
            let rhs = self.logic_and()?;
            expr = Expression::Logical(Box::new(expr), op, Box::new(rhs));
        }

        Ok(expr)
    }

    fn logic_and(&mut self) -> Result<Expression, ParseError> {
        let mut expr = self.equality()?;
        while self.match_token(&[TokenType::And]) {
            let op = self.previous();
            let rhs = self.equality()?;
            expr = Expression::Logical(Box::new(expr), op, Box::new(rhs));
        }

        Ok(expr)
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
                return Err(self.report_error(self.peek().clone(), "Failed to match rhs value"));
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

        if self.match_token(&[TokenType::Identifier]) {
            return Ok(Expression::Variable(self.previous()));
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
        self.error.had_error = true;

        ParseError
    }

    fn sync(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().typ == TokenType::Semicolon {
                return;
            }

            match self.peek().typ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }
}
