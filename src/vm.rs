use std::{cell::RefCell, rc::Rc};

use crate::{
    ast::{
        Assignment, Binary, BlockStatement, Grouping, Literal, LiteralValue, Logical, Node, Statement, Stmt, Unary,
        Variable,
    },
    environment::{Env, Environment},
    token::Token,
    visitor::{StatementVisitor, Visitor},
};

pub struct Vm {
    environment: Env,
}

#[derive(Debug)]
pub enum RuntimeError {
    ArgumentError(String),
    UnknownOperatorError(String),
    ZeroDivision(String),
    UndefinedVariable(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::ArgumentError(n) => write!(f, "{}", n),
            RuntimeError::UnknownOperatorError(s) => write!(f, "{}", s),
            RuntimeError::ZeroDivision(s) => write!(f, "{}", s),
            RuntimeError::UndefinedVariable(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Nil,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
        }
    }
}

impl std::ops::Neg for Value {
    type Output = Result<Value, RuntimeError>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Number(n) => Ok(Value::Number(-n)),
            other => Err(RuntimeError::ArgumentError(format!(
                "Expected number, but got {}",
                other
            ))),
        }
    }
}

impl std::ops::Sub for Value {
    type Output = Result<Value, RuntimeError>;

    fn sub(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
            (Value::Number(_l), other) => Err(RuntimeError::ArgumentError(format!(
                "Expected number, but got {}",
                other
            ))),
            (left, right) => Err(RuntimeError::ArgumentError(format!(
                "Invalid operands for -: {} and {}",
                left, right
            ))),
        }
    }
}

impl std::ops::Div for Value {
    type Output = Result<Value, RuntimeError>;

    fn div(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(l), Value::Number(0.0)) => {
                Err(RuntimeError::ZeroDivision(format!("Cannot divide {} by zero", l)))
            }
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
            (Value::Number(_l), other) => Err(RuntimeError::ArgumentError(format!(
                "Expected number, but got {}",
                other
            ))),
            (left, right) => Err(RuntimeError::ArgumentError(format!(
                "Invalid operands for /: {} and {}",
                left, right
            ))),
        }
    }
}

impl std::ops::Mul for Value {
    type Output = Result<Value, RuntimeError>;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
            (Value::Number(_l), other) => Err(RuntimeError::ArgumentError(format!(
                "Expected number, but got {}",
                other
            ))),
            (left, right) => Err(RuntimeError::ArgumentError(format!(
                "Invalid operands for *: {} and {}",
                left, right
            ))),
        }
    }
}

impl std::ops::Add for Value {
    type Output = Result<Value, RuntimeError>;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
            (Value::String(l), Value::String(r)) => Ok(Value::String(l + &r)),
            (Value::String(_l), other) => Err(RuntimeError::ArgumentError(format!(
                "Expected string, but got {}",
                other
            ))),
            (Value::Number(_l), other) => Err(RuntimeError::ArgumentError(format!(
                "Expected number, but got {}",
                other
            ))),
            (left, right) => Err(RuntimeError::ArgumentError(format!(
                "Invalid operands for +: {} and {}",
                left, right
            ))),
        }
    }
}

impl std::cmp::PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::Number(_l), Value::String(_r)) => false,
            (Value::Number(_l), Value::Boolean(_r)) => false,
            (Value::Number(_l), Value::Nil) => false,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::String(_l), Value::Number(_r)) => false,
            (Value::String(_l), Value::Boolean(_r)) => false,
            (Value::String(_l), Value::Nil) => false,
            (Value::Boolean(l), Value::Boolean(r)) => l == r,
            (Value::Boolean(_l), Value::Number(_r)) => false,
            (Value::Boolean(_l), Value::String(_r)) => false,
            (Value::Boolean(_l), Value::Nil) => false,
            (Value::Nil, Value::Nil) => true,
            (Value::Nil, Value::Number(_r)) => false,
            (Value::Nil, Value::String(_r)) => false,
            (Value::Nil, Value::Boolean(_r)) => false,
        }
    }
}

impl std::cmp::PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}

impl Vm {
    pub fn new() -> Self {
        Vm {
            environment: Environment::new_global(),
        }
    }

    fn truthy(&self, value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }

    fn execute_block(&mut self, block: &BlockStatement) -> Result<(), RuntimeError> {
        let previous = self.environment.clone();
        let inner = Rc::new(RefCell::new(Environment::new(Some(previous.clone()))));
        self.environment = inner;

        let result = block
            .statements
            .iter()
            .try_for_each(|statement| self.visit_statement(statement));

        self.environment = previous;
        result
    }
}

impl Visitor for Vm {
    type Output = Result<Value, RuntimeError>;

    fn visit_binary(&mut self, binary: &Binary) -> Self::Output {
        let left = binary.left.accept(self)?;
        let right = binary.right.accept(self)?;

        match *binary.operator {
            Token::Minus { line: _ } => Ok((left - right)?),
            Token::Slash { line: _ } => Ok((left / right)?),
            Token::Star { line: _ } => Ok((left * right)?),
            Token::Plus { line: _ } => Ok((left + right)?),
            Token::Greater { line: _ } => Ok(Value::Boolean(left > right)),
            Token::GreaterEqual { line: _ } => Ok(Value::Boolean(left >= right)),
            Token::Less { line: _ } => Ok(Value::Boolean(left < right)),
            Token::LessEqual { line: _ } => Ok(Value::Boolean(left <= right)),
            Token::BangEqual { line: _ } => Ok(Value::Boolean(left != right)),
            Token::EqualEqual { line: _ } => Ok(Value::Boolean(left == right)),
            _ => Err(RuntimeError::UnknownOperatorError(format!(
                "Unknown binary operator: {:?}",
                binary.operator
            ))),
        }
    }

    fn visit_variable(&mut self, variable: &Variable) -> Self::Output {
        match self.environment.borrow().get(&variable.token.value) {
            Ok(value) => Ok(value.clone()),
            Err(err) => Err(err),
        }
    }

    fn visit_assignment(&mut self, assignment: &Assignment) -> Self::Output {
        let value = assignment.value.accept(self)?;
        self.environment
            .borrow_mut()
            .assign(&assignment.name.value, value.clone())?;
        Ok(value)
    }

    fn visit_grouping(&mut self, grouping: &Grouping) -> Self::Output {
        grouping.expression.accept(self)
    }

    fn visit_literal(&mut self, literal: &Literal) -> Self::Output {
        match literal.value {
            LiteralValue::String(ref s) => Ok(Value::String(s.clone())),
            LiteralValue::Number(n) => Ok(Value::Number(n)),
            LiteralValue::Boolean(b) => Ok(Value::Boolean(b)),
            LiteralValue::Nil => Ok(Value::Nil),
        }
    }

    fn visit_logical(&mut self, logical: &Logical) -> Self::Output {
        let left = logical.left.accept(self)?;

        match *logical.operator {
            Token::Or { line: _ } => {
                if self.truthy(&left) {
                    Ok(left)
                } else {
                    logical.right.accept(self)
                }
            }
            _ => {
                if !self.truthy(&left) {
                    Ok(left)
                } else {
                    logical.right.accept(self)
                }
            }
        }
    }

    fn visit_unary(&mut self, unary: &Unary) -> Self::Output {
        let right = unary.right.accept(self)?;

        match *unary.operator {
            Token::Minus { line: _ } => -right,
            Token::Bang { line: _ } => Ok(Value::Boolean(!self.truthy(&right))),
            _ => Err(RuntimeError::UnknownOperatorError(format!(
                "Unknown unary operator: {:?}",
                unary.operator
            ))),
        }
    }
}

impl StatementVisitor for Vm {
    type Output = Result<(), RuntimeError>;

    fn visit_statement(&mut self, statement: &Statement) -> Self::Output {
        match statement {
            Statement::Expression(stmt) => {
                stmt.expression.accept(self)?;
                Ok(())
            }
            Statement::Print(stmt) => {
                let value = stmt.expression.accept(self)?;
                println!("{}", value);
                Ok(())
            }
            Statement::Variable(var) => {
                let value = var.value.accept(self)?;
                self.environment.borrow_mut().define(var.name.value.clone(), value);
                Ok(())
            }
            Statement::Block(block) => self.execute_block(block),
            Statement::If(if_stmt) => {
                let condition = if_stmt.condition.accept(self)?;

                if self.truthy(&condition) {
                    if_stmt.then_branch.accept(self)
                } else if let Some(else_branch) = &if_stmt.else_branch {
                    else_branch.accept(self)
                } else {
                    Ok(())
                }
            }
            Statement::While(while_stmt) => {
                loop {
                    let condition = while_stmt.condition.accept(self)?;
                    if !self.truthy(&condition) {
                        break;
                    }

                    while_stmt.body.accept(self)?;
                }

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Expr, ExpressionStatement, IfStatement, Stmt, VariableStatement, WhileStatement},
        token::Identifier,
    };

    use super::*;

    #[test]
    fn test_evaluating_literals() {
        let mut vm = Vm::new();
        let literal = Literal {
            value: LiteralValue::Number(42.0),
        };
        let result = literal.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(42.0));

        let literal = Literal {
            value: LiteralValue::String("Hello".to_string()),
        };
        let result = literal.accept(&mut vm).unwrap();
        assert_eq!(result, Value::String("Hello".to_string()));

        let bool = Literal {
            value: LiteralValue::Boolean(true),
        };
        let result = bool.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Boolean(true))
    }

    #[test]
    fn test_evaluating_unary() {
        let mut vm = Vm::new();
        let unary = Unary {
            operator: Box::new(Token::Minus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        };
        let result = unary.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(-42.0));

        let unary = Unary {
            operator: Box::new(Token::Bang { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Boolean(true),
            })),
        };
        let result = unary.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_evaluating_number_addition() {
        let mut vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
            operator: Box::new(Token::Plus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(58.0),
            })),
        };
        let result = binary.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_evaluating_string_addition() {
        let mut vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("Hello".to_string()),
            })),
            operator: Box::new(Token::Plus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String(" World".to_string()),
            })),
        };
        let result = binary.accept(&mut vm).unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_evaluating_invalid_addition() {
        let mut vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("Hello".to_string()),
            })),
            operator: Box::new(Token::Plus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        };
        assert!(binary.accept(&mut vm).is_err());

        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Boolean(false),
            })),
            operator: Box::new(Token::Plus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        };
        assert!(binary.accept(&mut vm).is_err());
    }

    #[test]
    fn test_evaluating_subtraction() {
        let mut vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.0),
            })),
            operator: Box::new(Token::Minus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(2.0),
            })),
        };
        let result = binary.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_evaluating_invalid_subtraction() {
        let mut vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("Hello".to_string()),
            })),
            operator: Box::new(Token::Minus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        };
        assert!(binary.accept(&mut vm).is_err());

        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Boolean(false),
            })),
            operator: Box::new(Token::Minus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        };
        assert!(binary.accept(&mut vm).is_err());
    }

    #[test]
    fn test_evaluating_division() {
        let mut vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.0),
            })),
            operator: Box::new(Token::Slash { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(2.0),
            })),
        };
        let result = binary.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(2.5));
    }

    #[test]
    fn test_evaluating_invalid_division() {
        let mut vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.5),
            })),
            operator: Box::new(Token::Slash { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("Hello".to_string()),
            })),
        };
        assert!(binary.accept(&mut vm).is_err());

        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.5),
            })),
            operator: Box::new(Token::Slash { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(0.0),
            })),
        };
        assert!(binary.accept(&mut vm).is_err());
    }

    #[test]
    fn test_evaluating_multiplication() {
        let mut vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.0),
            })),
            operator: Box::new(Token::Star { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(2.0),
            })),
        };
        let result = binary.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_evaluating_invalid_multiplication() {
        let mut vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.5),
            })),
            operator: Box::new(Token::Star { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("Hello".to_string()),
            })),
        };
        assert!(binary.accept(&mut vm).is_err());

        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.5),
            })),
            operator: Box::new(Token::Star { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Boolean(false),
            })),
        };
        assert!(binary.accept(&mut vm).is_err());
    }

    #[test]
    fn test_evaluating_global_variables() {
        let mut vm = Vm::new();

        let definition_statement = Statement::Variable(VariableStatement {
            name: Box::new(Identifier {
                value: "x".to_string(),
                line: 1,
            }),
            value: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        });

        definition_statement.accept(&mut vm).unwrap();

        let variable_expression = Expr::Variable(Variable {
            token: Box::new(Identifier {
                line: 1,
                value: "x".to_string(),
            }),
        });

        let result = variable_expression.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_evaluating_assignment() {
        let mut vm = Vm::new();

        let statements = vec![
            Statement::Variable(VariableStatement {
                name: Box::new(Identifier {
                    value: "x".to_string(),
                    line: 1,
                }),
                value: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Number(42.0),
                })),
            }),
            Statement::Expression(ExpressionStatement {
                expression: Box::new(Expr::Assignment(Assignment {
                    name: Box::new(Identifier {
                        value: "x".to_string(),
                        line: 1,
                    }),
                    value: Box::new(Expr::Literal(Literal {
                        value: LiteralValue::Number(10.0),
                    })),
                })),
            }),
        ];

        for statement in statements {
            statement.accept(&mut vm).unwrap();
        }

        let variable_expression = Expr::Variable(Variable {
            token: Box::new(Identifier {
                line: 1,
                value: "x".to_string(),
            }),
        });

        let result = variable_expression.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_if_statement() {
        let mut vm = Vm::new();

        let statements = vec![
            Statement::Variable(VariableStatement {
                name: Box::new(Identifier {
                    value: "x".to_string(),
                    line: 1,
                }),
                value: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Number(42.0),
                })),
            }),
            Statement::If(IfStatement {
                condition: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Boolean(true),
                })),
                then_branch: Box::new(Statement::Expression(ExpressionStatement {
                    expression: Box::new(Expr::Assignment(Assignment {
                        name: Box::new(Identifier {
                            value: "x".to_string(),
                            line: 1,
                        }),
                        value: Box::new(Expr::Literal(Literal {
                            value: LiteralValue::Number(10.0),
                        })),
                    })),
                })),
                else_branch: None,
            }),
        ];

        for statement in statements {
            statement.accept(&mut vm).unwrap();
        }
        let variable_expression = Expr::Variable(Variable {
            token: Box::new(Identifier {
                line: 1,
                value: "x".to_string(),
            }),
        });

        let result = variable_expression.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_if_statement_with_else() {
        let mut vm = Vm::new();

        let statements = vec![
            Statement::Variable(VariableStatement {
                name: Box::new(Identifier {
                    value: "x".to_string(),
                    line: 1,
                }),
                value: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Number(42.0),
                })),
            }),
            Statement::If(IfStatement {
                condition: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Boolean(false),
                })),
                then_branch: Box::new(Statement::Expression(ExpressionStatement {
                    expression: Box::new(Expr::Assignment(Assignment {
                        name: Box::new(Identifier {
                            value: "x".to_string(),
                            line: 1,
                        }),
                        value: Box::new(Expr::Literal(Literal {
                            value: LiteralValue::Number(10.0),
                        })),
                    })),
                })),
                else_branch: Some(Box::new(Statement::Expression(ExpressionStatement {
                    expression: Box::new(Expr::Assignment(Assignment {
                        name: Box::new(Identifier {
                            value: "x".to_string(),
                            line: 1,
                        }),
                        value: Box::new(Expr::Literal(Literal {
                            value: LiteralValue::Number(5.0),
                        })),
                    })),
                }))),
            }),
        ];

        for statement in statements {
            statement.accept(&mut vm).unwrap();
        }
        let variable_expression = Expr::Variable(Variable {
            token: Box::new(Identifier {
                line: 1,
                value: "x".to_string(),
            }),
        });

        let result = variable_expression.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_or_statement() {
        let mut vm = Vm::new();

        let statements = vec![Statement::Variable(VariableStatement {
            name: Box::new(Identifier {
                value: "x".to_string(),
                line: 1,
            }),
            value: Box::new(Expr::Logical(Logical {
                left: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Boolean(false),
                })),
                operator: Box::new(Token::Or { line: 1 }),
                right: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Number(5.0),
                })),
            })),
        })];

        for statement in statements {
            statement.accept(&mut vm).unwrap();
        }
        let variable_expression = Expr::Variable(Variable {
            token: Box::new(Identifier {
                line: 1,
                value: "x".to_string(),
            }),
        });

        let result = variable_expression.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_or_statement_short_circuit() {
        let mut vm = Vm::new();

        let statements = vec![Statement::Variable(VariableStatement {
            name: Box::new(Identifier {
                value: "x".to_string(),
                line: 1,
            }),
            value: Box::new(Expr::Logical(Logical {
                left: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Number(15.0),
                })),
                operator: Box::new(Token::Or { line: 1 }),
                right: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Number(5.0),
                })),
            })),
        })];

        for statement in statements {
            statement.accept(&mut vm).unwrap();
        }
        let variable_expression = Expr::Variable(Variable {
            token: Box::new(Identifier {
                line: 1,
                value: "x".to_string(),
            }),
        });

        let result = variable_expression.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(15.0));
    }

    #[test]
    fn test_and_statement() {
        let mut vm = Vm::new();

        let statements = vec![Statement::Variable(VariableStatement {
            name: Box::new(Identifier {
                value: "x".to_string(),
                line: 1,
            }),
            value: Box::new(Expr::Logical(Logical {
                left: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Boolean(true),
                })),
                operator: Box::new(Token::And { line: 1 }),
                right: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Number(5.0),
                })),
            })),
        })];

        for statement in statements {
            statement.accept(&mut vm).unwrap();
        }
        let variable_expression = Expr::Variable(Variable {
            token: Box::new(Identifier {
                line: 1,
                value: "x".to_string(),
            }),
        });

        let result = variable_expression.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_and_statement_short_circuit() {
        let mut vm = Vm::new();

        let statements = vec![Statement::Variable(VariableStatement {
            name: Box::new(Identifier {
                value: "x".to_string(),
                line: 1,
            }),
            value: Box::new(Expr::Logical(Logical {
                left: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Boolean(false),
                })),
                operator: Box::new(Token::And { line: 1 }),
                right: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Number(5.0),
                })),
            })),
        })];

        for statement in statements {
            statement.accept(&mut vm).unwrap();
        }
        let variable_expression = Expr::Variable(Variable {
            token: Box::new(Identifier {
                line: 1,
                value: "x".to_string(),
            }),
        });

        let result = variable_expression.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_while_loop() {
        let mut vm = Vm::new();

        let statements = vec![
            Statement::Variable(VariableStatement {
                name: Box::new(Identifier {
                    value: "x".to_string(),
                    line: 1,
                }),
                value: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::Number(0.0),
                })),
            }),
            Statement::While(WhileStatement {
                condition: Box::new(Expr::Binary(Binary {
                    left: Box::new(Expr::Variable(Variable {
                        token: Box::new(Identifier {
                            value: "x".to_string(),
                            line: 1,
                        }),
                    })),
                    operator: Box::new(Token::Less { line: 1 }),
                    right: Box::new(Expr::Literal(Literal {
                        value: LiteralValue::Number(5.0),
                    })),
                })),
                body: Box::new(Statement::Variable(VariableStatement {
                    name: Box::new(Identifier {
                        value: "x".to_string(),
                        line: 1,
                    }),
                    value: Box::new(Expr::Binary(Binary {
                        left: Box::new(Expr::Variable(Variable {
                            token: Box::new(Identifier {
                                value: "x".to_string(),
                                line: 1,
                            }),
                        })),
                        operator: Box::new(Token::Plus { line: 1 }),
                        right: Box::new(Expr::Literal(Literal {
                            value: LiteralValue::Number(1.0),
                        })),
                    })),
                })),
            }),
        ];

        for statement in statements {
            statement.accept(&mut vm).unwrap();
        }
        let variable_expression = Expr::Variable(Variable {
            token: Box::new(Identifier {
                line: 1,
                value: "x".to_string(),
            }),
        });

        let result = variable_expression.accept(&mut vm).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }
}
