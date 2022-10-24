use crate::expr::Expr;

pub fn print(expr: &Expr) -> String {
    match expr {
        Expr::Binary(left, op, right) => format!("({} {} {})", op.lexeme, print(&left), print(&right)),
        Expr::Unary(op, right) => format!("({} {})", op.lexeme, print(&right)),
        Expr::Literal(l) => format!("({:?})", l),
        Expr::Grouping(expr) => format!("(group {})", print(&expr)),
    }
}
