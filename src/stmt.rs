use std::rc::Rc;

use crate::expr::Expr;
use crate::token::Token;

#[derive(Clone)]
pub enum Stmt {
    Null,
    Block(Vec<Stmt>),
    Class(Token, Vec<Stmt>), // more specifically, will contain Stmt.Function
    Expression(Rc<Expr>),
    Function(Token, Vec<Token>, Vec<Stmt>),
    If(Rc<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Print(Rc<Expr>),
    Return(Token, Option<Rc<Expr>>),
    Var(Token, Option<Rc<Expr>>),
    While(Rc<Expr>, Box<Stmt>),
}
