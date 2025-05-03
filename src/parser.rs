use crate::{
    ast::{
        Binary, Expr, ExpressionStatement, Grouping, Literal, LiteralValue, PrintStatement,
        Statement, Unary,
    },
    token::Token,
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

    pub fn parse(&mut self) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();

        while let Some(token) = self.peek() {
            match token {
                Token::Eof => break,
                _ => {
                    statements.push(self.statement());
                }
            }
        }

        statements
    }

    fn statement(&mut self) -> Statement {
        match self.peek() {
            Some(Token::Print { line: _ }) => {
                self.advance();
                self.print_statement()
            }
            _ => self.expression_statement(),
        }
    }

    fn print_statement(&mut self) -> Statement {
        let value = self.expression();

        match self.peek() {
            Some(Token::Semicolon { line: _ }) => {
                self.advance();
            }
            _ => {
                self.errors.push(format!(
                    "[line {}] Error: Expected ';' after value.",
                    self.previous().unwrap().line()
                ));
            }
        }

        Statement::Print(PrintStatement {
            expression: Box::new(value),
        })
    }

    fn expression_statement(&mut self) -> Statement {
        let value = self.expression();

        match self.peek() {
            Some(Token::Semicolon { line: _ }) => {
                self.advance();
            }
            _ => {
                self.errors.push(format!(
                    "[line {}] Error: Expected ';' after value.",
                    self.previous().unwrap().line()
                ));
            }
        }

        Statement::Expression(ExpressionStatement {
            expression: Box::new(value),
        })
    }

    fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while let Some(token) = self.peek() {
            match token {
                Token::BangEqual { line: _ } | Token::EqualEqual { line: _ } => {
                    self.advance();
                }
                _ => break,
            }
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
        if let Some(token) = self.peek() {
            match token {
                Token::Eof => {}
                _ => {
                    self.current += 1;
                }
            }
        }

        self.previous()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current - 1)
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while let Some(token) = self.peek() {
            match token {
                Token::Greater { line: _ }
                | Token::GreaterEqual { line: _ }
                | Token::Less { line: _ }
                | Token::LessEqual { line: _ } => {
                    self.advance();
                }
                _ => break,
            }

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

        while let Some(token) = self.peek() {
            match token {
                Token::Minus { line: _ } | Token::Plus { line: _ } => {
                    self.advance();
                }
                _ => break,
            }

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

        while let Some(token) = self.peek() {
            match token {
                Token::Slash { line: _ } | Token::Star { line: _ } => {
                    self.advance();
                }
                _ => break,
            }

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
        match self.peek() {
            Some(Token::Bang { line: _ } | Token::Minus { line: _ }) => {
                self.advance();
                let operator = Box::new(self.previous().unwrap().clone());
                let right = self.unary();

                Expr::Unary(Unary {
                    operator,
                    right: Box::new(right),
                })
            }
            _ => self.primary(),
        }
    }

    fn primary(&mut self) -> Expr {
        match self.peek() {
            Some(Token::False { value, line: _ } | Token::True { value, line: _ }) => {
                let deref_value = *value;
                self.advance();
                return Expr::Literal(Literal {
                    value: LiteralValue::Boolean(deref_value),
                });
            }
            Some(Token::Nil { line: _ }) => {
                self.advance();
                return Expr::Literal(Literal {
                    value: LiteralValue::Nil,
                });
            }
            Some(Token::Number { value, line: _ }) => {
                let deref_value = *value;
                self.advance();
                return Expr::Literal(Literal {
                    value: LiteralValue::Number(deref_value),
                });
            }
            Some(Token::LeftParen { line: _ }) => {
                self.advance();
                let expr = Box::new(self.expression());

                match self.peek() {
                    Some(token) => match token {
                        Token::RightParen { line: _ } => {
                            self.advance();
                        }
                        other => {
                            self.errors.push(format!(
                                "[line {}] Error at '(': Expect ')' after expression.",
                                other.line()
                            ));
                        }
                    },
                    None => {
                        self.errors.push(format!(
                            "[line {}] Error: Expected ')' after expression.",
                            self.previous().unwrap().line()
                        ));
                    }
                }

                return Expr::Grouping(Grouping { expression: expr });
            }
            _ => {}
        }

        Expr::Literal(Literal {
            value: LiteralValue::Nil,
        })
    }

    // fn synchronize(&mut self) {
    //     self.advance();

    //     while !self.is_at_end() {
    //         if self.previous().unwrap().token_type == TokenType::Semicolon {
    //             return;
    //         }

    //         match self.peek().unwrap().token_type {
    //             TokenType::Class
    //             | TokenType::For
    //             | TokenType::Fun
    //             | TokenType::If
    //             | TokenType::While
    //             | TokenType::Print
    //             | TokenType::Return
    //             | TokenType::Var => {
    //                 return;
    //             }
    //             _ => {}
    //         }

    //         self.advance();
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing_print_statements() {
        let tokens = vec![
            Token::Print { line: 1 },
            Token::Number {
                value: 42.0,
                line: 1,
            },
            Token::Semicolon { line: 1 },
            Token::Eof,
        ];

        let mut errors = Vec::new();
        let mut parser = Parser::new(tokens, &mut errors);
        let statements = parser.parse();

        assert_eq!(statements.len(), 1);
        assert!(
            errors.is_empty(),
            "Expected no errors, but got: {:?}",
            errors
        );

        match &statements[0] {
            Statement::Print(_print) => {}
            _ => panic!("Expected a print statement."),
        }
    }

    #[test]
    fn test_parsing_expression_statements() {
        let tokens = vec![
            Token::Number {
                value: 42.0,
                line: 1,
            },
            Token::Semicolon { line: 1 },
            Token::Eof,
        ];

        let mut errors = Vec::new();
        let mut parser = Parser::new(tokens, &mut errors);
        let statements = parser.parse();

        assert_eq!(statements.len(), 1);
        assert!(
            errors.is_empty(),
            "Expected no errors, but got: {:?}",
            errors
        );

        match &statements[0] {
            Statement::Expression(_expr) => {}
            _ => panic!("Expected an expression statement."),
        }
    }

    #[test]
    fn test_parsing_errors_on_missing_semi_colons() {
        let tokens = vec![
            Token::Print { line: 1 },
            Token::Number {
                value: 42.0,
                line: 1,
            },
            Token::Eof,
        ];

        let mut errors = Vec::new();
        let mut parser = Parser::new(tokens, &mut errors);
        parser.parse();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0], "[line 1] Error: Expected ';' after value.");
    }
}
