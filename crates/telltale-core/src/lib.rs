//! Core definition of the Telltale language, including its syntax and semantics

/// A variable in the language that can contain any valid value (e.g. 'x', 'my_num')
#[derive(Debug)]
pub struct Variable(String);

#[derive(Debug)]
pub enum IntExpr {
    Const(usize),
    Var(Variable),
    Add(Box<Self>, Box<Self>),
    Mult(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
}

#[derive(Debug)]
pub enum BoolExpr {
    LessThan(IntExpr, IntExpr),
    Equal(IntExpr, IntExpr),
    // TODO: Add others: Not, And, Or, etc.
}

#[derive(Debug)]
pub enum Telltale {
    Assignment(Variable, IntExpr),
    Seq(Box<Self>, Box<Self>),
}
