use crate::expr::Expr;
use crate::token::Token;

pub enum Stmt {
    Null,
    Block(Vec<Stmt>),
    Expression(Expr),
    Print(Expr),
    Var(Token, Option<Expr>)
}
