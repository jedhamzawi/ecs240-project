//! Possible errors from the purity check

use std::fmt;

use crate::Variable;

#[derive(Debug, PartialEq)]
pub enum PurityError {
    /// A declared impure variable doesn't exist in global scope or as one of the function's
    /// parameters
    UnknownVariable {
        function_name: String,
        variable: Variable,
    },
    /// A declared impure variable is passed by value and cannot influence outer scope
    DeclaredVariableIsValue {
        function_name: String,
        variable: Variable,
    },
    /// The same variable appears more than once in the list.
    DuplicateVariable {
        function_name: String,
        variable: Variable,
    },
    /// A declared variable is never modified in the function body
    ///
    /// The variable may be read, but it is never assigned or passed to another
    /// function
    UnmodifiedVariable {
        function_name: String,
        variable: Variable,
    },
    // TODO: for now, unchecked/unimplemented
    // /// A declared variable is local to the function (including locals that
    // /// shadow a global of the same name)
    // LocalVariable {
    //     function_name: String,
    //     variable: Variable,
    // },
    // /// The function is marked impure but doesn't directly modify anything
    // /// outside its scope
    // ///
    // /// If the function calls *another* function that modifies outside of its
    // /// scope, this function isn't itself impure. If we called that impure,
    // /// then the impure designation would quickly become constant noise.
    // UnnecessaryImpure { function_name: String },
    // /// The function modifies outer state but has no impure marker
    // MissingImpure {
    //     function_name: String,
    //     modified: Vec<Variable>,
    // },
    // /// The function is marked impure, but its list leaves out a variable it modifies
    // ///
    // /// This error differs from [`Self::MissingImpure`] in that the marker exists
    // /// but is incomplete
    // MissingImpureVariable {
    //     function_name: String,
    //     variable: Variable,
    // },
    // /// The function calls an impure callee, but its own list doesn't include
    // /// the variables that callee modifies
    // UndeclaredTransitiveMutation {
    //     function_name: String,
    //     callee_name: String,
    //     variable: Variable,
    // },
    // /// The call target can't be resolved, so purity can't be determined.
    // UnknownFunction {
    //     function_name: String,
    //     callee_name: String,
    // },
}

impl fmt::Display for PurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PurityError::UnknownVariable {
                function_name,
                variable,
            } => write!(
                f,
                "function '{function_name}' declared impure variable '{variable}' \
                that isn't a parameter or a global variable"
            ),
            PurityError::DeclaredVariableIsValue {
                function_name,
                variable,
            } => write!(
                f,
                "function '{function_name}' declared impure variable '{variable}' \
                that is passed as a value, not a reference"
            ),
            PurityError::DuplicateVariable {
                function_name,
                variable,
            } => write!(
                f,
                "function '{function_name}' declared a duplicate impure variable '{variable}'"
            ),
            PurityError::UnmodifiedVariable {
                function_name,
                variable,
            } => write!(
                f,
                "function '{function_name}' declared impure variable '{variable}' \
                that isn't modified in the function body or passed to a function"
            ),
        }
    }
}

impl std::error::Error for PurityError {}
