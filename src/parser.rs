use crate::{
    ast::{Binary, Expr, Grouping, Literal, Unary},
    token::{Token, TokenType},
};

pub struct Parser<'a> {
    current: usize,
    tokens: Vec<Token>,
    errors: &'a mut Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, errors: &'a mut Vec<String>) -> Self {
        Parser {
            current: 0,
            tokens,
            errors,
        }
    }

    pub fn parse(&mut self) -> Expr {
        self.expression()
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_token(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = Box::new(self.previous().unwrap().clone());
            let right = self.comparison();

            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn match_token(&mut self, types: Vec<TokenType>) -> bool {
        let matches = types.iter().any(|&token_type| self.check(token_type));
        if matches {
            self.advance();
        }
        matches
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        match self.peek() {
            Some(token) => token.token_type == token_type,
            None => false,
        }
    }

    fn is_at_end(&self) -> bool {
        match self.peek() {
            Some(token) => token.token_type == TokenType::Eof,
            None => true,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_token(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = Box::new(self.previous().unwrap().clone());
            let right = self.term();

            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_token(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = Box::new(self.previous().unwrap().clone());
            let right = self.factor();

            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_token(vec![TokenType::Slash, TokenType::Star]) {
            let operator = Box::new(self.previous().unwrap().clone());
            let right = self.unary();

            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_token(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = Box::new(self.previous().unwrap().clone());
            let right = self.unary();

            return Expr::Unary(Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Expr {
        if self.match_token(vec![TokenType::False]) {
            return Expr::Literal(Literal {
                value: "false".to_string(),
            });
        }

        if self.match_token(vec![TokenType::True]) {
            return Expr::Literal(Literal {
                value: "true".to_string(),
            });
        }

        if self.match_token(vec![TokenType::Nil]) {
            return Expr::Literal(Literal {
                value: "null".to_string(),
            });
        }

        if self.match_token(vec![TokenType::Number, TokenType::String]) {
            return Expr::Literal(Literal {
                value: self.previous().unwrap().lexeme.clone(),
            });
        }

        if self.match_token(vec![TokenType::LeftParen]) {
            let expr = Box::new(self.expression());
            self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Expr::Grouping(Grouping { expression: expr });
        }

        Expr::Literal(Literal {
            value: "null".to_string(),
        })
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Option<&Token> {
        if self.check(token_type) {
            self.advance()
        } else {
            let token = self.peek().unwrap();
            self.errors.push(format!(
                "[line {}] Error at '{}': {}",
                token.line, token.lexeme, message
            ));
            None
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().unwrap().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().unwrap().token_type {
                TokenType::Class
                | TokenType::For
                | TokenType::Fun
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return
                | TokenType::Var => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }
}
