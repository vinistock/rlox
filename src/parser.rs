use crate::{
    ast::{
        Assignment, Binary, BlockStatement, Expr, ExpressionStatement, Grouping, IfStatement, Literal, LiteralValue,
        Logical, PrintStatement, Statement, Unary, Variable, VariableStatement, WhileStatement,
    },
    token::Token,
};

pub enum ParseError {
    ExpectedTokenError(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ExpectedTokenError(msg) => write!(f, "{}", msg),
        }
    }
}

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
                _ => match self.declaration() {
                    Ok(statement) => statements.push(statement),
                    Err(e) => {
                        self.errors.push(format!("{}", e));
                        self.synchronize();
                    }
                },
            }
        }

        statements
    }

    fn declaration(&mut self) -> Result<Statement, ParseError> {
        match self.peek() {
            Some(Token::Var { line: _ }) => {
                self.advance();
                self.var_declaration()
            }
            _ => self.statement(),
        }
    }

    fn var_declaration(&mut self) -> Result<Statement, ParseError> {
        let identifier = match self.advance() {
            Some(Token::Identifier(token)) => Ok(token.clone()),
            other => Err(ParseError::ExpectedTokenError(format!(
                "[line {}] Error: Expected variable name.",
                other.unwrap().line()
            ))),
        }?;

        let initializer = match self.peek() {
            Some(Token::Equal { line: _ }) => {
                self.advance();
                Ok(self.expression())
            }
            _ => Err(ParseError::ExpectedTokenError(format!(
                "[line {}] Error: Expected '=' after variable name.",
                identifier.line
            ))),
        }?;

        match self.peek() {
            Some(Token::Semicolon { line: _ }) => {
                self.advance();
                Ok(Statement::Variable(VariableStatement {
                    name: Box::new(identifier),
                    value: Box::new(initializer),
                }))
            }
            _ => Err(ParseError::ExpectedTokenError(format!(
                "[line {}] Error: Expected ';' after variable declaration.",
                identifier.line
            ))),
        }
    }

    fn statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek() {
            Some(Token::If { line: _ }) => {
                self.advance();
                self.if_statement()
            }
            Some(Token::Print { line: _ }) => {
                self.advance();
                self.print_statement()
            }
            Some(Token::While { line: _ }) => {
                self.advance();
                self.while_statement()
            }
            Some(Token::LeftBrace { line: _ }) => {
                self.advance();
                self.block()
            }
            _ => self.expression_statement(),
        }
    }

    fn while_statement(&mut self) -> Result<Statement, ParseError> {
        if let Some(Token::LeftParen { line: _ }) = self.peek() {
            self.advance();
            let condition = self.expression();

            if let Some(Token::RightParen { line: _ }) = self.peek() {
                self.advance();
                let body = Box::new(self.statement()?);

                Ok(Statement::While(WhileStatement {
                    condition: Box::new(condition),
                    body,
                }))
            } else {
                let message = format!(
                    "[line {}] Error: Expected ')' after while condition.",
                    self.previous().unwrap().line()
                );
                self.errors.push(message.clone());
                Err(ParseError::ExpectedTokenError(message))
            }
        } else {
            let message = format!(
                "[line {}] Error: Expected '(' after 'while'.",
                self.previous().unwrap().line()
            );
            self.errors.push(message.clone());
            Err(ParseError::ExpectedTokenError(message))
        }
    }

    fn block(&mut self) -> Result<Statement, ParseError> {
        let mut statements = Vec::new();

        while let Some(token) = self.peek() {
            match token {
                Token::RightBrace { line: _ } => {
                    self.advance();
                    break;
                }
                Token::Eof => {
                    return Err(ParseError::ExpectedTokenError(format!(
                        "[line {}] Error: Expected '}}' after block, but found EOF",
                        self.previous().unwrap().line()
                    )));
                }
                _ => {
                    let statement = self.declaration()?;
                    statements.push(statement);
                }
            }
        }

        Ok(Statement::Block(BlockStatement { statements }))
    }

    fn print_statement(&mut self) -> Result<Statement, ParseError> {
        let value = self.expression();

        match self.peek() {
            Some(Token::Semicolon { line: _ }) => {
                self.advance();

                Ok(Statement::Print(PrintStatement {
                    expression: Box::new(value),
                }))
            }
            _ => {
                let message = format!(
                    "[line {}] Error: Expected ';' after value.",
                    self.previous().unwrap().line()
                );
                self.errors.push(message.clone());
                Err(ParseError::ExpectedTokenError(message))
            }
        }
    }

    fn expression_statement(&mut self) -> Result<Statement, ParseError> {
        let value = self.expression();

        match self.peek() {
            Some(Token::Semicolon { line: _ }) => {
                self.advance();
                Ok(Statement::Expression(ExpressionStatement {
                    expression: Box::new(value),
                }))
            }
            _ => {
                let message = format!(
                    "[line {}] Error: Expected ';' after value.",
                    self.previous().unwrap().line()
                );
                self.errors.push(message.clone());
                Err(ParseError::ExpectedTokenError(message))
            }
        }
    }

    fn if_statement(&mut self) -> Result<Statement, ParseError> {
        if let Some(Token::LeftParen { line: _ }) = self.peek() {
            self.advance();
            let condition = self.expression();

            if let Some(Token::RightParen { line: _ }) = self.peek() {
                self.advance();
                let then_branch = Box::new(self.statement()?);
                let else_branch = if let Some(Token::Else { line: _ }) = self.peek() {
                    self.advance();
                    Some(Box::new(self.statement()?))
                } else {
                    None
                };

                Ok(Statement::If(IfStatement {
                    condition: Box::new(condition),
                    then_branch,
                    else_branch,
                }))
            } else {
                let message = format!(
                    "[line {}] Error: Expected ')' after if condition.",
                    self.previous().unwrap().line()
                );
                self.errors.push(message.clone());
                Err(ParseError::ExpectedTokenError(message))
            }
        } else {
            let message = format!(
                "[line {}] Error: Expected '(' after 'if'.",
                self.previous().unwrap().line()
            );
            self.errors.push(message.clone());
            Err(ParseError::ExpectedTokenError(message))
        }
    }

    fn assignment(&mut self) -> Expr {
        let expression = self.or();

        if let Some(Token::Equal { line: _ }) = self.peek() {
            self.advance();
            let value = self.assignment();

            if let Expr::Variable(variable) = expression {
                return Expr::Assignment(Assignment {
                    name: variable.token,
                    value: Box::new(value),
                });
            } else {
                self.errors.push(format!(
                    "[line {}] Error: Invalid assignment target.",
                    self.previous().unwrap().line()
                ));
            }
        }

        expression
    }

    fn or(&mut self) -> Expr {
        let mut expr = self.and();

        while let Some(token) = self.peek() {
            match token {
                Token::Or { line: _ } => {
                    self.advance();
                }
                _ => break,
            }
            let operator = Box::new(self.previous().unwrap().clone());
            let right = self.and();

            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn and(&mut self) -> Expr {
        let mut expr = self.equality();

        while let Some(token) = self.peek() {
            match token {
                Token::And { line: _ } => {
                    self.advance();
                }
                _ => break,
            }
            let operator = Box::new(self.previous().unwrap().clone());
            let right = self.equality();

            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        expr
    }

    fn expression(&mut self) -> Expr {
        self.assignment()
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
            Some(Token::Identifier(token)) => {
                let variable_expr = Expr::Variable(Variable {
                    token: Box::new(token.clone()),
                });
                self.advance();
                return variable_expr;
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

    fn synchronize(&mut self) {
        self.advance();

        while let Some(token) = self.peek() {
            if let Some(Token::Semicolon { line: _ }) = self.previous() {
                break;
            }

            match token {
                Token::Eof
                | Token::Class { line: _ }
                | Token::Fun { line: _ }
                | Token::Var { line: _ }
                | Token::For { line: _ }
                | Token::If { line: _ }
                | Token::While { line: _ }
                | Token::Print { line: _ }
                | Token::Return { line: _ } => break,
                _ => {}
            }

            self.advance();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::token::Identifier;

    use super::*;

    #[test]
    fn test_parsing_print_statements() {
        let tokens = vec![
            Token::Print { line: 1 },
            Token::Number { value: 42.0, line: 1 },
            Token::Semicolon { line: 1 },
            Token::Eof,
        ];

        let mut errors = Vec::new();
        let mut parser = Parser::new(tokens, &mut errors);
        let statements = parser.parse();

        assert_eq!(statements.len(), 1);
        assert!(errors.is_empty(), "Expected no errors, but got: {:?}", errors);

        match &statements[0] {
            Statement::Print(_print) => {}
            _ => panic!("Expected a print statement."),
        }
    }

    #[test]
    fn test_parsing_expression_statements() {
        let tokens = vec![
            Token::Number { value: 42.0, line: 1 },
            Token::Semicolon { line: 1 },
            Token::Eof,
        ];

        let mut errors = Vec::new();
        let mut parser = Parser::new(tokens, &mut errors);
        let statements = parser.parse();

        assert_eq!(statements.len(), 1);
        assert!(errors.is_empty(), "Expected no errors, but got: {:?}", errors);

        match &statements[0] {
            Statement::Expression(_expr) => {}
            _ => panic!("Expected an expression statement."),
        }
    }

    #[test]
    fn test_parsing_errors_on_missing_semi_colons() {
        let tokens = vec![
            Token::Print { line: 1 },
            Token::Number { value: 42.0, line: 1 },
            Token::Eof,
        ];

        let mut errors = Vec::new();
        let mut parser = Parser::new(tokens, &mut errors);
        parser.parse();

        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0], "[line 1] Error: Expected ';' after value.");
    }

    #[test]
    fn test_parsing_a_print_statement() {
        let tokens = vec![
            Token::Print { line: 1 },
            Token::Identifier(Identifier {
                value: "x".to_string(),
                line: 1,
            }),
            Token::Plus { line: 1 },
            Token::Identifier(Identifier {
                value: "y".to_string(),
                line: 1,
            }),
            Token::Semicolon { line: 1 },
            Token::Eof,
        ];

        let mut errors = Vec::new();
        let mut parser = Parser::new(tokens, &mut errors);
        let result = parser.parse();

        assert_eq!(errors.len(), 0, "Expected no errors, but got: {:?}", errors);
        assert_eq!(result.len(), 1);

        match &result[0] {
            Statement::Print(print_stmt) => match *print_stmt.expression {
                Expr::Binary(ref binary) => {
                    match *binary.left {
                        Expr::Variable(ref var) => {
                            assert_eq!(
                                var.token,
                                Box::new(Identifier {
                                    value: "x".to_string(),
                                    line: 1
                                })
                            );
                        }
                        _ => panic!("Expected a variable expression."),
                    }
                    assert_eq!(binary.operator, Box::new(Token::Plus { line: 1 }));
                    match *binary.right {
                        Expr::Variable(ref var) => {
                            assert_eq!(
                                var.token,
                                Box::new(Identifier {
                                    value: "y".to_string(),
                                    line: 1
                                })
                            );
                        }
                        _ => panic!("Expected a variable expression."),
                    }
                }
                _ => panic!("Expected a literal expression."),
            },
            _ => panic!("Expected a print statement."),
        }
    }

    #[test]
    fn test_parsing_assignments() {
        let tokens = vec![
            Token::Identifier(Identifier {
                value: "x".to_string(),
                line: 1,
            }),
            Token::Equal { line: 1 },
            Token::Number { value: 42.0, line: 1 },
            Token::Semicolon { line: 1 },
            Token::Eof,
        ];

        let mut errors = Vec::new();
        let mut parser = Parser::new(tokens, &mut errors);
        let result = parser.parse();

        assert_eq!(errors.len(), 0, "Expected no errors, but got: {:?}", errors);
        assert_eq!(result.len(), 1);

        match &result[0] {
            Statement::Expression(expr) => match &*expr.expression {
                Expr::Assignment(assignment) => {
                    assert_eq!(assignment.name.value, "x");

                    match &*assignment.value {
                        Expr::Literal(literal) => match &literal.value {
                            LiteralValue::Number(value) => {
                                assert_eq!(*value, 42.0);
                            }
                            _ => panic!("Expected a number literal."),
                        },
                        _ => panic!("Expected a literal expression."),
                    }
                }
                _ => panic!("Expected an assignment expression."),
            },
            _ => panic!("Expected an expression statement."),
        }
    }

    #[test]
    fn test_parsing_shadowed_assignments() {
        let tokens = vec![
            Token::Var { line: 1 },
            Token::Identifier(Identifier {
                value: "x".to_string(),
                line: 1,
            }),
            Token::Equal { line: 1 },
            Token::Number { value: 42.0, line: 1 },
            Token::Semicolon { line: 1 },
            Token::LeftBrace { line: 2 },
            Token::Var { line: 3 },
            Token::Identifier(Identifier {
                value: "x".to_string(),
                line: 3,
            }),
            Token::Equal { line: 3 },
            Token::Number { value: 30.0, line: 3 },
            Token::Semicolon { line: 3 },
            Token::RightBrace { line: 4 },
            Token::Eof,
        ];

        let mut errors = Vec::new();
        let mut parser = Parser::new(tokens, &mut errors);
        let result = parser.parse();

        assert_eq!(errors.len(), 0, "Expected no errors, but got: {:?}", errors);
        assert_eq!(result.len(), 2);
    }
}
