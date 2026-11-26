//! Purity checking for Telltale

use crate::{Argument, Function, GlobalAssignment, Parameter, Statement, Telltale, Variable};

mod error;

pub use error::PurityError;

impl Telltale {
    /// Check whether a function's impure designation is semantically
    /// valid according to Telltale
    ///
    /// If the check fails, the check returns a specific error
    /// containing information on the offending function
    // TODO: should we return a list of all offending functions?
    pub fn check_purity(&self) -> Result<(), PurityError> {
        for function in self.functions.values() {
            function.check_purity(&self.globals)?;
        }
        Ok(())
    }
}

/// Find the first repeat in a slice
// TODO: we should check if this is really slow
fn first_repeat<T: PartialEq>(v: &[T]) -> Option<&T> {
    v.iter()
        .enumerate()
        .find(|(i, x)| v[..*i].contains(x))
        .map(|(_, x)| x)
}

impl Function {
    /// Check whether a function's impure designation is semantically
    /// valid according to Telltale
    ///
    /// If the check fails, the check returns a specific error
    /// containing information on the offending element
    fn check_purity(&self, globals: &[GlobalAssignment]) -> Result<(), PurityError> {
        // check that all variables are actual parameters or globals
        if let Some(missing_var) = self.impure_designation.as_ref().and_then(|designation| {
            designation.variables.iter().find(|var| {
                // check the variable is a parameter
                !self.parameters.iter().any(|param| param.is_variable(var))
                    // if it's not a parameter, check if it's a global
                    && !globals.iter().any(|global| global.assigns(var))
            })
        }) {
            return Err(PurityError::UnknownVariable {
                function_name: self.name.clone(),
                variable: missing_var.clone(),
            });
        }
        // check that no variables are values;
        // variables passed as value to a function can't modify outside scope by definition
        if let Some(value_var) = self.impure_designation.as_ref().and_then(|designation| {
            designation.variables.iter().find(|var| {
                self.parameters
                    .iter()
                    .any(|param| matches!(param, Parameter::Value(_)) && param.is_variable(var))
            })
        }) {
            return Err(PurityError::DeclaredVariableIsValue {
                function_name: self.name.clone(),
                variable: value_var.clone(),
            });
        }
        // check that the impure designation doesn't have repeated variables
        if let Some(repeat_var) = self
            .impure_designation
            .as_ref()
            .and_then(|designation| first_repeat(&designation.variables))
        {
            return Err(PurityError::DuplicateVariable {
                function_name: self.name.clone(),
                variable: repeat_var.clone(),
            });
        }
        // check that there are no variables that are declared but are definitely
        // not modified
        if let Some(unmodified_var) = self.impure_designation.as_ref().and_then(|designation| {
            designation.variables.iter().find(|var| {
                !self
                    .body
                    .statements
                    .iter()
                    .any(|statement| statement.assigns_or_passes(var))
            })
        }) {
            return Err(PurityError::UnmodifiedVariable {
                function_name: self.name.clone(),
                variable: unmodified_var.clone(),
            });
        }
        Ok(())
    }
}

impl Parameter {
    /// Returns `true` if the parameter is a reference to the
    /// given variable
    fn is_variable(&self, v: &Variable) -> bool {
        match &self {
            Parameter::Value(variable) => variable == v,
            Parameter::Reference(variable) => variable == v,
        }
    }
}

impl GlobalAssignment {
    /// Returns `true` if the global is the given variable
    fn assigns(&self, v: &Variable) -> bool {
        &self.0 == v
    }
}

impl Statement {
    /// Returns `true` if the statement assigns the given variable
    /// or passes it to another function that may mutate it
    fn assigns_or_passes(&self, v: &Variable) -> bool {
        match &self {
            Statement::Assignment(variable, _) => variable == v,
            Statement::FunctionCall(function_call) => function_call
                .arguments
                .iter()
                .any(|arg| matches!(arg, Argument::Reference(arg_v) if v == arg_v)),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn unknown_var() {
        let globals = Vec::new();
        let function_name = "unknown_var";
        let function = Function {
            name: function_name.to_string(),
            impure_designation: Some(ImpureDesignation {
                variables: vec![Variable::new("x"), Variable::new("y")],
            }),
            parameters: Vec::new(),
            body: Body {
                statements: vec![Statement::Declaration(Variable::new("x"), None)],
            },
        };
        let result = function.check_purity(&globals);
        assert!(result.is_err());
        let err = result.expect_err("purity check did not output error");
        assert_eq!(
            err,
            PurityError::UnknownVariable {
                function_name: function_name.to_string(),
                variable: Variable::new("x")
            }
        );
        let function = Function {
            name: function_name.to_string(),
            impure_designation: Some(ImpureDesignation {
                variables: vec![Variable::new("x"), Variable::new("y")],
            }),
            parameters: vec![Parameter::Reference(Variable::new("x"))],
            body: Body {
                statements: Vec::new(),
            },
        };
        let result = function.check_purity(&globals);
        assert!(result.is_err());
        let err = result.expect_err("purity check did not output error");
        assert_eq!(
            err,
            PurityError::UnknownVariable {
                function_name: function_name.to_string(),
                variable: Variable::new("y")
            }
        );
    }

    #[test]
    fn value_var() {
        let globals = Vec::new();
        let function_name = "value_var";
        let function = Function {
            name: function_name.to_string(),
            impure_designation: Some(ImpureDesignation {
                variables: vec![Variable::new("x"), Variable::new("y")],
            }),
            parameters: vec![
                Parameter::Reference(Variable::new("x")),
                Parameter::Value(Variable::new("y")),
            ],
            body: Body {
                statements: vec![Statement::Declaration(Variable::new("x"), None)],
            },
        };
        let result = function.check_purity(&globals);
        assert!(result.is_err());
        let err = result.expect_err("purity check did not output error");
        assert_eq!(
            err,
            PurityError::DeclaredVariableIsValue {
                function_name: function_name.to_string(),
                variable: Variable::new("y")
            }
        );
        let function = Function {
            name: function_name.to_string(),
            impure_designation: Some(ImpureDesignation {
                variables: vec![Variable::new("x"), Variable::new("y")],
            }),
            parameters: vec![
                Parameter::Value(Variable::new("x")),
                Parameter::Value(Variable::new("y")),
            ],
            body: Body {
                statements: Vec::new(),
            },
        };
        let result = function.check_purity(&globals);
        assert!(result.is_err());
        let err = result.expect_err("purity check did not output error");
        assert_eq!(
            err,
            PurityError::DeclaredVariableIsValue {
                function_name: function_name.to_string(),
                variable: Variable::new("x")
            }
        );
    }

    #[test]
    fn duplicate_var() {
        let globals = Vec::new();
        let function_name = "duplicate_var";
        let function = Function {
            name: function_name.to_string(),
            impure_designation: Some(ImpureDesignation {
                variables: vec![Variable::new("x"), Variable::new("x")],
            }),
            parameters: vec![Parameter::Reference(Variable::new("x"))],
            body: Body {
                statements: Vec::new(),
            },
        };
        let result = function.check_purity(&globals);
        assert!(result.is_err());
        let err = result.expect_err("purity check did not output error");
        assert_eq!(
            err,
            PurityError::DuplicateVariable {
                function_name: function_name.to_string(),
                variable: Variable::new("x")
            }
        );
        let function = Function {
            name: function_name.to_string(),
            impure_designation: Some(ImpureDesignation {
                variables: vec![
                    Variable::new("x"),
                    Variable::new("y"),
                    Variable::new("x"),
                    Variable::new("y"),
                ],
            }),
            parameters: vec![
                Parameter::Reference(Variable::new("x")),
                Parameter::Reference(Variable::new("y")),
            ],
            body: Body {
                statements: Vec::new(),
            },
        };
        let result = function.check_purity(&globals);
        assert!(result.is_err());
        let err = result.expect_err("purity check did not output error");
        assert_eq!(
            err,
            PurityError::DuplicateVariable {
                function_name: function_name.to_string(),
                variable: Variable::new("x")
            }
        );
        let function = Function {
            name: function_name.to_string(),
            impure_designation: Some(ImpureDesignation {
                variables: vec![Variable::new("x"), Variable::new("y"), Variable::new("z")],
            }),
            parameters: vec![
                Parameter::Reference(Variable::new("x")),
                Parameter::Reference(Variable::new("y")),
                Parameter::Reference(Variable::new("z")),
            ],
            body: Body {
                statements: Vec::new(),
            },
        };
        let result = function.check_purity(&globals);
        assert!(
            result.is_ok()
                || result.is_err_and(|err| !matches!(
                    err,
                    PurityError::DuplicateVariable {
                        function_name: _,
                        variable: _
                    }
                )),
            "output duplicate variable error when it shouldn't have"
        );
    }

    #[test]
    fn unmodified_var() {
        let globals = Vec::new();
        let function_name = "unmodified_var";
        let function = Function {
            name: function_name.to_string(),
            impure_designation: Some(ImpureDesignation {
                variables: vec![Variable::new("x")],
            }),
            parameters: vec![Parameter::Reference(Variable::new("x"))],
            body: Body {
                statements: Vec::new(),
            },
        };
        let result = function.check_purity(&globals);
        assert!(result.is_err());
        let err = result.expect_err("purity check did not output error");
        assert_eq!(
            err,
            PurityError::UnmodifiedVariable {
                function_name: function_name.to_string(),
                variable: Variable::new("x")
            }
        );
        let function = Function {
            name: function_name.to_string(),
            impure_designation: Some(ImpureDesignation {
                variables: vec![Variable::new("x")],
            }),
            parameters: vec![Parameter::Reference(Variable::new("x"))],
            body: Body {
                statements: vec![Statement::Assignment(
                    Variable::new("x"),
                    IntExpr::Const(10),
                )],
            },
        };
        let result = function.check_purity(&globals);
        assert!(
            result.is_ok()
                || result.is_err_and(|err| !matches!(
                    err,
                    PurityError::UnmodifiedVariable {
                        function_name: _,
                        variable: _
                    }
                )),
            "output unmodified variable error when it shouldn't have"
        );
        let function = Function {
            name: function_name.to_string(),
            impure_designation: Some(ImpureDesignation {
                variables: vec![Variable::new("x")],
            }),
            parameters: vec![Parameter::Reference(Variable::new("x"))],
            body: Body {
                statements: vec![Statement::FunctionCall(FunctionCall {
                    function_name: "some_func".to_string(),
                    arguments: vec![Argument::Reference(Variable::new("x"))],
                })],
            },
        };
        let result = function.check_purity(&globals);
        assert!(
            result.is_ok()
                || result.is_err_and(|err| !matches!(
                    err,
                    PurityError::UnmodifiedVariable {
                        function_name: _,
                        variable: _
                    }
                )),
            "output unmodified variable error when it shouldn't have"
        );
    }
}
