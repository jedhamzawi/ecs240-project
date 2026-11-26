//! The Telltale interpreter that directly executes a [`Telltale`] program

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use telltale_core::{
    Argument, Body, Function, FunctionCall, IntExpr, MAIN_FN_NAME, Parameter, Statement, Telltale,
};

type Slot = Rc<RefCell<Option<usize>>>;
type Scope = HashMap<String, Slot>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpretError {
    MissingMainFunction,
    UndefinedFunction(String),
    UndefinedVariable(String),
    UninitializedVariable(String),
    ArityMismatch {
        function_name: String,
        expected: usize,
        found: usize,
    },
    ReferenceRequiresVariable,
    ValueExpectedForReferenceParam(String),
    DivisionByZero,
}

impl fmt::Display for InterpretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMainFunction => {
                write!(f, "program is missing '{}' function", MAIN_FN_NAME)
            }
            Self::UndefinedFunction(name) => write!(f, "undefined function '{}'", name),
            Self::UndefinedVariable(name) => write!(f, "undefined variable '{}'", name),
            Self::UninitializedVariable(name) => {
                write!(f, "use of uninitialized variable '{}'", name)
            }
            Self::ArityMismatch {
                function_name,
                expected,
                found,
            } => write!(
                f,
                "function '{}' expected {} argument(s), found {}",
                function_name, expected, found
            ),
            Self::ReferenceRequiresVariable => {
                write!(f, "reference argument '&' must refer to a variable")
            }
            Self::ValueExpectedForReferenceParam(param) => write!(
                f,
                "reference parameter '{}' requires a '&<var>' argument",
                param
            ),
            Self::DivisionByZero => write!(f, "attempt to divide by zero"),
        }
    }
}

impl std::error::Error for InterpretError {}

enum ExecOutcome {
    Continue,
    Return(usize),
}

#[derive(Debug, Default)]
pub struct TelltaleInterpreter {
    globals: Scope,
}

impl TelltaleInterpreter {
    pub fn new() -> Self {
        Self::default()
    }

    /// execute a Telltale program  at main
    pub fn interpret(&mut self, prog: &Telltale) {
        self.interpret_with_return(prog)
            .unwrap_or_else(|err| panic!("Runtime error while interpreting program: {err}"));
    }

    /// execute a Telltale program starting at main and return its exit/return value
    pub fn interpret_with_return(&mut self, prog: &Telltale) -> Result<usize, InterpretError> {
        self.globals.clear();

        // initialize global variables in declaration order
        for global in &prog.globals {
            let val = self.eval_int_expr(&global.1, &[], &self.globals)?;
            let slot = Rc::new(RefCell::new(Some(val)));
            self.globals.insert(global.0.name.clone(), slot);
        }

        let main_fn = prog
            .functions
            .get(MAIN_FN_NAME)
            .ok_or(InterpretError::MissingMainFunction)?;

        if !main_fn.parameters.is_empty() {
            return Err(InterpretError::ArityMismatch {
                function_name: MAIN_FN_NAME.to_string(),
                expected: 0,
                found: main_fn.parameters.len(),
            });
        }

        self.call_function(MAIN_FN_NAME, &[], &prog.functions, &mut Vec::new())
    }

    fn resolve_slot<'a>(
        name: &str,
        call_stack: &'a [Scope],
        globals: &'a Scope,
    ) -> Option<&'a Slot> {
        for scope in call_stack.iter().rev() {
            if let Some(slot) = scope.get(name) {
                return Some(slot);
            }
        }
        globals.get(name)
    }

    fn read_var(
        name: &str,
        call_stack: &[Scope],
        globals: &Scope,
    ) -> Result<usize, InterpretError> {
        let slot = Self::resolve_slot(name, call_stack, globals)
            .ok_or_else(|| InterpretError::UndefinedVariable(name.to_string()))?;
        slot.borrow()
            .ok_or_else(|| InterpretError::UninitializedVariable(name.to_string()))
    }

    fn write_var(
        name: &str,
        value: usize,
        call_stack: &[Scope],
        globals: &Scope,
    ) -> Result<(), InterpretError> {
        let slot = Self::resolve_slot(name, call_stack, globals)
            .ok_or_else(|| InterpretError::UndefinedVariable(name.to_string()))?;
        *slot.borrow_mut() = Some(value);
        Ok(())
    }

    fn eval_int_expr(
        &self,
        expr: &IntExpr,
        call_stack: &[Scope],
        globals: &Scope,
    ) -> Result<usize, InterpretError> {
        match expr {
            IntExpr::Const(n) => Ok(*n),
            IntExpr::Var(var) => Self::read_var(&var.name, call_stack, globals),
            IntExpr::Add(lhs, rhs) => {
                let a = self.eval_int_expr(lhs, call_stack, globals)?;
                let b = self.eval_int_expr(rhs, call_stack, globals)?;
                Ok(a.wrapping_add(b))
            }
            IntExpr::Sub(lhs, rhs) => {
                let a = self.eval_int_expr(lhs, call_stack, globals)?;
                let b = self.eval_int_expr(rhs, call_stack, globals)?;
                Ok(a.wrapping_sub(b))
            }
            IntExpr::Mult(lhs, rhs) => {
                let a = self.eval_int_expr(lhs, call_stack, globals)?;
                let b = self.eval_int_expr(rhs, call_stack, globals)?;
                Ok(a.wrapping_mul(b))
            }
            IntExpr::Div(lhs, rhs) => {
                let a = self.eval_int_expr(lhs, call_stack, globals)?;
                let b = self.eval_int_expr(rhs, call_stack, globals)?;
                if b == 0 {
                    return Err(InterpretError::DivisionByZero);
                }
                Ok(a / b)
            }
        }
    }

    fn call_function(
        &mut self,
        function_name: &str,
        args: &[Argument],
        functions: &HashMap<String, Function>,
        call_stack: &mut Vec<Scope>,
    ) -> Result<usize, InterpretError> {
        let (param_count, is_main) = {
            let func = functions
                .get(function_name)
                .ok_or_else(|| InterpretError::UndefinedFunction(function_name.to_string()))?;
            (func.parameters.len(), func.name == MAIN_FN_NAME)
        };

        if is_main && !args.is_empty() {
            return Err(InterpretError::ArityMismatch {
                function_name: function_name.to_string(),
                expected: 0,
                found: args.len(),
            });
        }

        if param_count != args.len() {
            return Err(InterpretError::ArityMismatch {
                function_name: function_name.to_string(),
                expected: param_count,
                found: args.len(),
            });
        }

        // evaluate value arguments and bind reference parameters in the caller's environment
        let mut frame: Scope = HashMap::new();

        {
            let func = functions
                .get(function_name)
                .ok_or_else(|| InterpretError::UndefinedFunction(function_name.to_string()))?;

            for (param, arg) in func.parameters.iter().zip(args.iter()) {
                match (param, arg) {
                    (Parameter::Value(param_var), Argument::IntExpr(expr)) => {
                        let val = self.eval_int_expr(expr, call_stack, &self.globals)?;
                        frame.insert(param_var.name.clone(), Rc::new(RefCell::new(Some(val))));
                    }
                    (Parameter::Reference(param_var), Argument::Reference(arg_var)) => {
                        let target_slot =
                            Self::resolve_slot(&arg_var.name, call_stack, &self.globals)
                                .ok_or_else(|| {
                                    InterpretError::UndefinedVariable(arg_var.name.clone())
                                })?
                                .clone();
                        frame.insert(param_var.name.clone(), target_slot);
                    }
                    (Parameter::Reference(param_var), Argument::IntExpr(_)) => {
                        return Err(InterpretError::ValueExpectedForReferenceParam(
                            param_var.name.clone(),
                        ));
                    }
                    (Parameter::Value(_), Argument::Reference(_)) => {
                        return Err(InterpretError::ReferenceRequiresVariable);
                    }
                }
            }
        }

        call_stack.push(frame);
        let exec_res = {
            let body = &functions
                .get(function_name)
                .ok_or_else(|| InterpretError::UndefinedFunction(function_name.to_string()))?
                .body;
            self.exec_body(body, functions, call_stack)
        };
        call_stack.pop();

        match exec_res? {
            ExecOutcome::Return(val) => Ok(val),
            ExecOutcome::Continue => Ok(0),
        }
    }

    fn exec_body(
        &mut self,
        body: &Body,
        functions: &HashMap<String, Function>,
        call_stack: &mut Vec<Scope>,
    ) -> Result<ExecOutcome, InterpretError> {
        for stmt in &body.statements {
            match self.exec_statement(stmt, functions, call_stack)? {
                ExecOutcome::Continue => {}
                ret @ ExecOutcome::Return(_) => return Ok(ret),
            }
        }
        Ok(ExecOutcome::Continue)
    }

    fn exec_statement(
        &mut self,
        stmt: &Statement,
        functions: &HashMap<String, Function>,
        call_stack: &mut Vec<Scope>,
    ) -> Result<ExecOutcome, InterpretError> {
        match stmt {
            Statement::Declaration(var, init_expr) => {
                let initial_val = match init_expr {
                    Some(expr) => Some(self.eval_int_expr(expr, call_stack, &self.globals)?),
                    None => None,
                };
                let current_scope = call_stack
                    .last_mut()
                    .expect("active stack frame must exist during statement execution");
                current_scope.insert(var.name.clone(), Rc::new(RefCell::new(initial_val)));
                Ok(ExecOutcome::Continue)
            }

            Statement::Assignment(var, expr) => {
                let val = self.eval_int_expr(expr, call_stack, &self.globals)?;
                Self::write_var(&var.name, val, call_stack, &self.globals)?;
                Ok(ExecOutcome::Continue)
            }

            Statement::Return(expr) => {
                let val = self.eval_int_expr(expr, call_stack, &self.globals)?;
                Ok(ExecOutcome::Return(val))
            }

            Statement::FunctionCall(FunctionCall {
                function_name,
                arguments,
            }) => {
                self.exec_function_call(function_name, arguments, functions, call_stack)?;
                Ok(ExecOutcome::Continue)
            }
        }
    }

    fn exec_function_call(
        &mut self,
        function_name: &str,
        arguments: &[Argument],
        functions: &HashMap<String, Function>,
        call_stack: &mut Vec<Scope>,
    ) -> Result<usize, InterpretError> {
        if function_name == "print" {
            for (i, arg) in arguments.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                match arg {
                    Argument::IntExpr(expr) => {
                        let val = self.eval_int_expr(expr, call_stack, &self.globals)?;
                        print!("{val}");
                    }
                    Argument::Reference(var) => {
                        let val = Self::read_var(&var.name, call_stack, &self.globals)?;
                        print!("{val}");
                    }
                }
            }
            println!();
            return Ok(0);
        }

        if !functions.contains_key(function_name) {
            return Err(InterpretError::UndefinedFunction(function_name.to_string()));
        }

        self.call_function(function_name, arguments, functions, call_stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use telltale_core::{
        Body, Function, FunctionCall, IntExpr, Parameter, Statement, Telltale, Variable,
    };

    #[test]
    fn test_reference_parameter_mutates_caller_variable() {
        let mut functions = HashMap::new();
        functions.insert(
            "add_to".to_string(),
            Function {
                name: "add_to".to_string(),
                impure_designation: None,
                parameters: vec![
                    Parameter::Reference(Variable::new("target")),
                    Parameter::Value(Variable::new("delta")),
                ],
                body: Body {
                    statements: vec![Statement::Assignment(
                        Variable::new("target"),
                        IntExpr::Add(
                            Box::new(IntExpr::Var(Variable::new("target"))),
                            Box::new(IntExpr::Var(Variable::new("delta"))),
                        ),
                    )],
                },
            },
        );
        functions.insert(
            MAIN_FN_NAME.to_string(),
            Function {
                name: MAIN_FN_NAME.to_string(),
                impure_designation: None,
                parameters: vec![],
                body: Body {
                    statements: vec![
                        Statement::Declaration(Variable::new("x"), Some(IntExpr::Const(10))),
                        Statement::FunctionCall(FunctionCall {
                            function_name: "add_to".to_string(),
                            arguments: vec![
                                Argument::Reference(Variable::new("x")),
                                Argument::IntExpr(IntExpr::Const(5)),
                            ],
                        }),
                        Statement::Return(IntExpr::Var(Variable::new("x"))),
                    ],
                },
            },
        );

        let prog = Telltale {
            functions,
            globals: Vec::new(),
        };

        let mut interp = TelltaleInterpreter::new();
        let result = interp.interpret_with_return(&prog).unwrap();
        assert_eq!(result, 15);
    }

    #[test]
    fn test_value_parameter_does_not_mutate_caller_variable() {
        let mut functions = HashMap::new();
        functions.insert(
            "inc_copy".to_string(),
            Function {
                name: "inc_copy".to_string(),
                impure_designation: None,
                parameters: vec![Parameter::Value(Variable::new("x"))],
                body: Body {
                    statements: vec![
                        Statement::Assignment(
                            Variable::new("x"),
                            IntExpr::Add(
                                Box::new(IntExpr::Var(Variable::new("x"))),
                                Box::new(IntExpr::Const(1)),
                            ),
                        ),
                        Statement::Return(IntExpr::Var(Variable::new("x"))),
                    ],
                },
            },
        );
        functions.insert(
            MAIN_FN_NAME.to_string(),
            Function {
                name: MAIN_FN_NAME.to_string(),
                impure_designation: None,
                parameters: vec![],
                body: Body {
                    statements: vec![
                        Statement::Declaration(Variable::new("x"), Some(IntExpr::Const(10))),
                        Statement::FunctionCall(FunctionCall {
                            function_name: "inc_copy".to_string(),
                            arguments: vec![Argument::IntExpr(IntExpr::Var(Variable::new("x")))],
                        }),
                        Statement::Return(IntExpr::Var(Variable::new("x"))),
                    ],
                },
            },
        );

        let prog = Telltale {
            functions,
            globals: Vec::new(),
        };

        let mut interp = TelltaleInterpreter::new();
        let result = interp.interpret_with_return(&prog).unwrap();
        assert_eq!(result, 10);
    }
}
