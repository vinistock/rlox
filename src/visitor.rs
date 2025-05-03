use crate::ast::{Binary, Grouping, Literal, LiteralValue, Node, Statement, Unary};

pub trait Visitor {
    type Output;
    fn visit_binary(&self, binary: &Binary) -> Self::Output;
    fn visit_grouping(&self, grouping: &Grouping) -> Self::Output;
    fn visit_literal(&self, literal: &Literal) -> Self::Output;
    fn visit_unary(&self, unary: &Unary) -> Self::Output;
}

pub trait StatementVisitor {
    type Output;
    fn visit_statement(&self, statement: &Statement) -> Self::Output;
}

pub struct AstPrinter;

impl Visitor for AstPrinter {
    type Output = String;

    fn visit_binary(&self, binary: &Binary) -> Self::Output {
        format!(
            "({} {} {})",
            binary.operator.lexeme(),
            binary.left.accept(self),
            binary.right.accept(self)
        )
    }

    fn visit_grouping(&self, grouping: &Grouping) -> Self::Output {
        format!("(group {})", grouping.expression.accept(self))
    }

    fn visit_literal(&self, literal: &Literal) -> Self::Output {
        match literal.value {
            LiteralValue::String(ref s) => s.clone(),
            LiteralValue::Number(ref n) => n.to_string(),
            LiteralValue::Boolean(ref b) => b.to_string(),
            LiteralValue::Nil => "nil".to_string(),
        }
    }

    fn visit_unary(&self, unary: &Unary) -> Self::Output {
        format!("({} {})", unary.operator.lexeme(), unary.right.accept(self))
    }
}

impl StatementVisitor for AstPrinter {
    type Output = String;

    fn visit_statement(&self, statement: &Statement) -> Self::Output {
        match statement {
            Statement::Expression(expr) => expr.expression.accept(self),
            Statement::Print(print_stmt) => format!("print {}", print_stmt.expression.accept(self)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;
    use crate::token::Token;

    #[test]
    fn test_ast_printer() {
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("5".to_string()),
            })),
            operator: Box::new(Token::Plus { line: 1 }),
            right: Box::new(Expr::Literal(Literal {
                value: LiteralValue::String("3".to_string()),
            })),
        };

        let printer = AstPrinter;
        assert_eq!(printer.visit_binary(&binary), "(+ 5 3)".to_string());
    }

    #[test]
    fn test_ast_printer_more_complex_case() {
        let expr = Binary {
            left: Box::new(Expr::Unary(Unary {
                operator: Box::new(Token::Minus { line: 1 }),
                right: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::String("123".to_string()),
                })),
            })),
            operator: Box::new(Token::Star { line: 1 }),
            right: Box::new(Expr::Grouping(Grouping {
                expression: Box::new(Expr::Literal(Literal {
                    value: LiteralValue::String("45.67".to_string()),
                })),
            })),
        };

        let printer = AstPrinter;
        assert_eq!(
            printer.visit_binary(&expr),
            "(* (- 123) (group 45.67))".to_string()
        );
    }
}
