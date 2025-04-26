use crate::ast::{Binary, Grouping, Literal, Node, Unary};

pub trait Visitor {
    type Output;
    fn visit_binary(&self, binary: &Binary) -> Self::Output;
    fn visit_grouping(&self, binary: &Grouping) -> Self::Output;
    fn visit_literal(&self, binary: &Literal) -> Self::Output;
    fn visit_unary(&self, binary: &Unary) -> Self::Output;
}

pub struct AstPrinter;

impl Visitor for AstPrinter {
    type Output = String;
    fn visit_binary(&self, binary: &Binary) -> Self::Output {
        format!(
            "({} {} {})",
            binary.operator.lexeme,
            binary.left.accept(self),
            binary.right.accept(self)
        )
    }

    fn visit_grouping(&self, grouping: &Grouping) -> Self::Output {
        format!("(group {})", grouping.expression.accept(self))
    }

    fn visit_literal(&self, literal: &Literal) -> Self::Output {
        literal.value.clone()
    }

    fn visit_unary(&self, unary: &Unary) -> Self::Output {
        format!("({} {})", unary.operator.lexeme, unary.right.accept(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Expr;
    use crate::token::{Token, TokenType};

    #[test]
    fn test_ast_printer() {
        let binary = Binary {
            left: Box::new(Expr::Literal(Literal {
                value: "5".to_string(),
            })),
            operator: Token {
                token_type: TokenType::Plus,
                lexeme: "+".to_string(),
                line: 1,
            },
            right: Box::new(Expr::Literal(Literal {
                value: "3".to_string(),
            })),
        };

        let printer = AstPrinter;
        assert_eq!(printer.visit_binary(&binary), "(+ 5 3)".to_string());
    }

    #[test]
    fn test_ast_printer_more_complex_case() {
        let expr = Binary {
            left: Box::new(Expr::Unary(Unary {
                operator: Token {
                    token_type: TokenType::Minus,
                    lexeme: "-".to_string(),
                    line: 1,
                },
                right: Box::new(Expr::Literal(Literal {
                    value: "123".to_string(),
                })),
            })),
            operator: Token {
                token_type: TokenType::Star,
                lexeme: "*".to_string(),
                line: 1,
            },
            right: Box::new(Expr::Grouping(Grouping {
                expression: Box::new(Expr::Literal(Literal {
                    value: "45.67".to_string(),
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
