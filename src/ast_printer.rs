use crate::expr::Expr;
use Expr::*;

pub fn print(expr: &Expr) -> String {
    match expr {
        Binary(left, op, right) => format!("({} {} {})", op.lexeme, print(&left), print(&right)),
        Unary(op, right) => format!("({} {})", op.lexeme, print(&right)),
        Literal(literal) => format!("({})", literal.lexeme),
        Grouping(expr) => format!("(group {})", print(&expr))
    }
}