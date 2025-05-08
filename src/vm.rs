use crate::{
    ast::{Binary, Grouping, Literal, LiteralValue, Node, Statement, Unary, Variable},
    environment::Environment,
    token::Token,
    visitor::{StatementVisitor, Visitor},
};

pub struct Vm {
    environment: Environment,
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
            (Value::Number(l), Value::Number(0.0)) => Err(RuntimeError::ZeroDivision(format!(
                "Cannot divide {} by zero",
                l
            ))),
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
            environment: Environment::new(),
        }
    }

    fn truthy(&self, value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            _ => true,
        }
    }
}

impl Visitor for Vm {
    type Output = Result<Value, RuntimeError>;

    fn visit_binary(&self, binary: &Binary) -> Self::Output {
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

    fn visit_variable(&self, variable: &Variable) -> Self::Output {
        match self.environment.get(&variable.token.value) {
            Ok(value) => Ok(value.clone()),
            Err(err) => Err(err),
        }
    }

    fn visit_grouping(&self, grouping: &Grouping) -> Self::Output {
        grouping.expression.accept(self)
    }

    fn visit_literal(&self, literal: &Literal) -> Self::Output {
        match literal.value {
            LiteralValue::String(ref s) => Ok(Value::String(s.clone())),
            LiteralValue::Number(n) => Ok(Value::Number(n)),
            LiteralValue::Boolean(b) => Ok(Value::Boolean(b)),
            LiteralValue::Nil => Ok(Value::Nil),
        }
    }

    fn visit_unary(&self, unary: &Unary) -> Self::Output {
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
            Statement::Expression(expr) => {
                expr.expression.accept(self)?;
                Ok(())
            }
            Statement::Print(expr) => {
                let value = expr.expression.accept(self)?;
                println!("{}", value);
                Ok(())
            }
            Statement::Variable(var) => {
                let value = var.value.accept(self)?;
                self.environment.define(var.name.value.clone(), value);
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        ast::{Expr, Stmt, VariableStatement},
        token::Identifier,
    };

    use super::*;

    #[test]
    fn test_evaluating_literals() {
        let vm = Vm::new();
        let literal = Literal {
            value: LiteralValue::Number(42.0),
        };
        let result = literal.accept(&vm).unwrap();
        assert_eq!(result, Value::Number(42.0));

        let literal = Literal {
            value: LiteralValue::String("Hello".to_string()),
        };
        let result = literal.accept(&vm).unwrap();
        assert_eq!(result, Value::String("Hello".to_string()));

        let bool = Literal {
            value: LiteralValue::Boolean(true),
        };
        let result = bool.accept(&vm).unwrap();
        assert_eq!(result, Value::Boolean(true))
    }

    #[test]
    fn test_evaluating_unary() {
        let vm = Vm::new();
        let unary = Unary {
            operator: Box::new(Token::Minus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        };
        let result = unary.accept(&vm).unwrap();
        assert_eq!(result, Value::Number(-42.0));

        let unary = Unary {
            operator: Box::new(Token::Bang { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Boolean(true),
            })),
        };
        let result = unary.accept(&vm).unwrap();
        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_evaluating_number_addition() {
        let vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
            operator: Box::new(Token::Plus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(58.0),
            })),
        };
        let result = binary.accept(&vm).unwrap();
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_evaluating_string_addition() {
        let vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("Hello".to_string()),
            })),
            operator: Box::new(Token::Plus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String(" World".to_string()),
            })),
        };
        let result = binary.accept(&vm).unwrap();
        assert_eq!(result, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_evaluating_invalid_addition() {
        let vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("Hello".to_string()),
            })),
            operator: Box::new(Token::Plus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        };
        assert!(binary.accept(&vm).is_err());

        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Boolean(false),
            })),
            operator: Box::new(Token::Plus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        };
        assert!(binary.accept(&vm).is_err());
    }

    #[test]
    fn test_evaluating_subtraction() {
        let vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.0),
            })),
            operator: Box::new(Token::Minus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(2.0),
            })),
        };
        let result = binary.accept(&vm).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_evaluating_invalid_subtraction() {
        let vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("Hello".to_string()),
            })),
            operator: Box::new(Token::Minus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        };
        assert!(binary.accept(&vm).is_err());

        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Boolean(false),
            })),
            operator: Box::new(Token::Minus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(42.0),
            })),
        };
        assert!(binary.accept(&vm).is_err());
    }

    #[test]
    fn test_evaluating_division() {
        let vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.0),
            })),
            operator: Box::new(Token::Slash { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(2.0),
            })),
        };
        let result = binary.accept(&vm).unwrap();
        assert_eq!(result, Value::Number(2.5));
    }

    #[test]
    fn test_evaluating_invalid_division() {
        let vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.5),
            })),
            operator: Box::new(Token::Slash { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("Hello".to_string()),
            })),
        };
        assert!(binary.accept(&vm).is_err());

        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.5),
            })),
            operator: Box::new(Token::Slash { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(0.0),
            })),
        };
        assert!(binary.accept(&vm).is_err());
    }

    #[test]
    fn test_evaluating_multiplication() {
        let vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.0),
            })),
            operator: Box::new(Token::Star { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(2.0),
            })),
        };
        let result = binary.accept(&vm).unwrap();
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_evaluating_invalid_multiplication() {
        let vm = Vm::new();
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.5),
            })),
            operator: Box::new(Token::Star { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("Hello".to_string()),
            })),
        };
        assert!(binary.accept(&vm).is_err());

        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Number(5.5),
            })),
            operator: Box::new(Token::Star { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::Boolean(false),
            })),
        };
        assert!(binary.accept(&vm).is_err());
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

        let result = variable_expression.accept(&vm).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }
}
