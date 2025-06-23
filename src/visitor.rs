use crate::ast::{
    Assignment, Binary, Grouping, Literal, LiteralValue, Node, Statement, Unary, Variable,
};

pub trait Visitor {
    type Output;
    fn visit_binary(&mut self, binary: &Binary) -> Self::Output;
    fn visit_grouping(&mut self, grouping: &Grouping) -> Self::Output;
    fn visit_literal(&mut self, literal: &Literal) -> Self::Output;
    fn visit_unary(&mut self, unary: &Unary) -> Self::Output;
    fn visit_variable(&mut self, variable: &Variable) -> Self::Output;
    fn visit_assignment(&mut self, assignment: &Assignment) -> Self::Output;
}

pub trait StatementVisitor {
    type Output;
    fn visit_statement(&mut self, statement: &Statement) -> Self::Output;
}

pub struct AstPrinter;

impl Visitor for AstPrinter {
    type Output = String;

    fn visit_assignment(&mut self, assignment: &Assignment) -> Self::Output {
        format!("{} = {}", assignment.name, assignment.value.accept(self))
    }

    fn visit_binary(&mut self, binary: &Binary) -> Self::Output {
        format!(
            "({} {} {})",
            binary.operator.lexeme(),
            binary.left.accept(self),
            binary.right.accept(self)
        )
    }

    fn visit_variable(&mut self, variable: &Variable) -> Self::Output {
        variable.token.value.clone()
    }

    fn visit_grouping(&mut self, grouping: &Grouping) -> Self::Output {
        format!("(group {})", grouping.expression.accept(self))
    }

    fn visit_literal(&mut self, literal: &Literal) -> Self::Output {
        match literal.value {
            LiteralValue::String(ref s) => s.clone(),
            LiteralValue::Number(ref n) => n.to_string(),
            LiteralValue::Boolean(ref b) => b.to_string(),
            LiteralValue::Nil => "nil".to_string(),
        }
    }

    fn visit_unary(&mut self, unary: &Unary) -> Self::Output {
        format!("({} {})", unary.operator.lexeme(), unary.right.accept(self))
    }
}

impl StatementVisitor for AstPrinter {
    type Output = String;

    fn visit_statement(&mut self, statement: &Statement) -> Self::Output {
        match statement {
            Statement::Expression(expr) => expr.expression.accept(self),
            Statement::Print(print_stmt) => format!("print {}", print_stmt.expression.accept(self)),
            Statement::Variable(variable) => {
                format!("{}={}", variable.name, variable.value.accept(self))
            }
            Statement::Block(block) => {
                let mut result = "{".to_string();
                for stmt in &block.statements {
                    result.push_str(&self.visit_statement(stmt));
                    result.push_str(";\n");
                }
                result.push('}');
                result
            }
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

        let mut printer = AstPrinter;
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

        let mut printer = AstPrinter;
        assert_eq!(
            printer.visit_binary(&expr),
            "(* (- 123) (group 45.67))".to_string()
        );
    }
}
