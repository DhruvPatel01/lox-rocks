use crate::expr::Expr;
use crate::token::Token;

pub enum Stmt {
    Null,
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>)
}
