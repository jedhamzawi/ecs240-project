//! Core definition of the Telltale language, including its syntax and semantics

use std::{collections::HashMap, fmt};

mod purity;

/// The name of the main function in Telltale, the entrypoint function that is
/// executed when running the program
///
/// For any function in Telltale, if there is no execution path from main to
/// that function, that function is "dead code" and can never be run. 'main'
/// is the one and only entrypoint for a Telltale program.
pub const MAIN_FN_NAME: &str = "main";

/// A variable in the language that can contain any valid value (e.g. 'x', 'my_num')
#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: String,
}

impl Variable {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, PartialEq)]
pub enum IntExpr {
    /// A integer literal (e.g. '10')
    Const(usize),
    /// A variable containing an integer (e.g. 'x = 10; x')
    Var(Variable),
    /// Adding two integer expressions (e.g. 'x + 10')
    Add(Box<Self>, Box<Self>),
    /// Multiplying two integer expressions (e.g. 'x * 10')
    Mult(Box<Self>, Box<Self>),
    /// Subtracting two integer expressions (e.g. 'x - 10')
    Sub(Box<Self>, Box<Self>),
    /// Dividing two integer expressions using integer division
    /// (see <https://doc.rust-lang.org/std/ops/trait.Div.html>)
    /// (e.g. 'x / 10')
    Div(Box<Self>, Box<Self>),
}

// TODO: not used yet; we have no conditionals
#[derive(Debug, PartialEq)]
pub enum BoolExpr {
    LessThan(IntExpr, IntExpr),
    Equal(IntExpr, IntExpr),
    // TODO: Add others: Not, And, Or, etc.
}

// TODO: should this be a separate type..? Maybe not. Should references be
// IntExpr? Are references IntExpr? I think no
/// An argument to a function call
#[derive(Debug, PartialEq)]
pub enum Argument {
    /// An integer expression
    /// '10 + 3', 'x', 'x * y', etc.
    IntExpr(IntExpr),
    /// A reference to a variable
    /// '&x'
    Reference(Variable),
}

#[derive(Debug, PartialEq)]
pub struct FunctionCall {
    // TODO: different type?
    /// The name of the function being called
    pub function_name: String,
    /// The arguments passed to the function
    pub arguments: Vec<Argument>,
}

/// A single statement in the program, ending in a semicolon ';'
#[derive(Debug, PartialEq)]
pub enum Statement {
    // TODO: should each of these variants be structs with named fields?
    /// Declaring a new variable, optionally assigning it a value
    /// 'var x;' or 'var x = 10;'
    Declaration(Variable, Option<IntExpr>),
    /// Assigning a pre-existing variable
    /// 'x = 10;'
    Assignment(Variable, IntExpr),
    /// A return statement
    /// 'return x;'
    // TODO: don't worry about multiple return statements
    Return(IntExpr),
    /// A function call
    /// 'my_func(10)'
    // TODO: function name as a different type instad of string?
    // TODO: always pass mutuable references in compiler!!!!!!!
    FunctionCall(FunctionCall),
}

/// The body of a function: empty, a statement, or
/// a sequence of statements with an optional expression
/// at the end as the return value
#[derive(Debug, PartialEq, Default)]
pub struct Body {
    // TODO: can we just use a vec?
    pub statements: Vec<Statement>,
    // TODO: could we instead just have this as syntactic sugar
    // and just change the final statement to be a return of whatever
    // IntExpr is there?
    // TODO: need a lower level probably where it's just Vec<Statement>
    // TODO: do we add syntactic sugar for return expression?
    // pub return_expr: Option<IntExpr>,
}

/// A parameter to a function, either a value or a reference
///
/// Currently, parameters are *always* mutable
#[derive(Debug, PartialEq)]
pub enum Parameter {
    /// Passing the variable as a value, *moving* or *copying* the variable
    /// into the function under Rust semantics
    Value(Variable),
    /// Passing the variable
    Reference(Variable),
}

/// The `impure()` designation for a function, defining
/// that a function is impure and enumerating the elements
/// of outside state that may be mutated
#[derive(Debug, PartialEq)]
pub struct ImpureDesignation {
    /// The variables outside of this function's state that
    /// may be mutated
    // TODO: support references?
    pub variables: Vec<Variable>,
}

/// A function in Telltale, taking the form
/// 'fn my_func(x, y) { <BODY> }' or for impure
/// functions 'fn impure(x) my_func(x, y) { <BODY> }'
#[derive(Debug, PartialEq)]
pub struct Function {
    // TODO: should this be Variable? Or a different type?
    /// The name of the function
    pub name: String,
    // TODO: separation of pure/impure by something more clear?
    // an enum, generic, etc.? Otherwise checking whether a function
    // is pure is func.impure_designation.is_some(), which is fine;
    // we could just have a function is_pure() or something to be
    // more clear
    /// The designation of impurity for this function, required if
    /// the function is *actually* impure
    pub impure_designation: Option<ImpureDesignation>,
    /// Any parameters to the function
    // TODO: support references
    pub parameters: Vec<Parameter>,
    /// The body of the function
    pub body: Body,
}

/// A global assignment outside of a function's scope
///
/// This is distinct from a [`Statement::Declaration`] because
/// it is a declaration where the [`IntExpr`] is *required*. You cannot have
/// an uninitialized global variable. It's distinct from a
/// [`Statement::Assignment`] because this isn't an assignment, but
/// rather a declaration of a global variable followed by an assignment.
///
/// Examples:
///
/// 'var x = 10;'
/// 'var y = x;'
/// 'var z = 20 + 10;'
// TODO: could we represent this instead as two part sequence, the first declaring
// the variable, the second assigning it?
#[derive(Debug, PartialEq)]
pub struct GlobalAssignment(Variable, IntExpr);

/// A full, valid Telltale program
#[derive(Debug, PartialEq)]
pub struct Telltale {
    // TODO: hashmap? functions must be unique; this would make it
    // easier for the parser or other components to ensure uniqueness,
    // and for the interpreter and compiler to easily detect whether
    // a function call is misspelled or doesn't refer to a defined function
    /// The map of function names to functions in the program
    ///
    /// Every Telltale program *must* have at least the 'main' function
    /// with no parameters
    pub functions: HashMap<String, Function>,
    /// The program's global variables, variables that are declared outside
    /// of any function's scope and can be mutated and accessed by any
    /// function.
    pub globals: Vec<GlobalAssignment>,
}

impl Default for Telltale {
    /// Return an empty Telltale program, containing a single pure
    /// 'main' function with no body
    fn default() -> Self {
        Self {
            functions: HashMap::from_iter(std::iter::once((
                MAIN_FN_NAME.to_string(),
                Function {
                    name: "main".to_string(),
                    impure_designation: None,
                    parameters: Vec::new(),
                    body: Body::default(),
                },
            ))),
            globals: Vec::new(),
        }
    }
}
