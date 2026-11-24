//! Telltale parser

use std::collections::HashMap;

use telltale_core::{
    Argument, Body, Function, FunctionCall, ImpureDesignation, IntExpr, MAIN_FN_NAME, Parameter,
    Statement, Telltale, Variable,
};

mod error;
pub use error::TelltaleParseError;

// 1. TOKENS

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Fn,
    Var,
    Return,
    Impure,

    Identifier(String),
    Number(usize),
    Reference,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    Comma,
    Semicolon,
    Equal,

    Plus,
    Minus,
    Star,
    Slash,
}

// 2. TOKENIZER

fn tokenize(input: &str) -> Result<Vec<Token>, TelltaleParseError> {
    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(c) = chars.next() {
        // Ignore whitespace
        if c.is_whitespace() {
            continue;
        }

        // Ignore single-line comments
        if c == '/' && chars.peek() == Some(&'/') {
            chars.next();

            while let Some(ch) = chars.next() {
                if ch == '\n' {
                    break;
                }
            }

            continue;
        }

        // Read integer literals
        if c.is_ascii_digit() {
            let mut number = String::new();
            number.push(c);

            while let Some(&next) = chars.peek() {
                if next.is_ascii_digit() {
                    number.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            let value = number
                .parse::<usize>()
                .map_err(|_| TelltaleParseError::InvalidNumber)?;

            tokens.push(Token::Number(value));
            continue;
        }

        // Read identifiers and keywords
        if c.is_ascii_alphabetic() || c == '_' {
            let mut word = String::new();
            word.push(c);

            while let Some(&next) = chars.peek() {
                if next.is_ascii_alphanumeric() || next == '_' {
                    word.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            let token = match word.as_str() {
                "fn" => Token::Fn,
                "var" => Token::Var,
                "return" => Token::Return,
                "impure" => Token::Impure,
                _ => Token::Identifier(word),
            };

            tokens.push(token);
            continue;
        }

        // Read symbols
        let token = match c {
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            '{' => Token::LeftBrace,
            '}' => Token::RightBrace,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '=' => Token::Equal,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '&' => Token::Reference,
            _ => return Err(TelltaleParseError::UnexpectedCharacter(c)),
        };

        tokens.push(token);
    }

    Ok(tokens)
}

// Our tokenizer scans the source code character by character. It recognizes keywords, identifiers, integer literals, operators, punctuation, whitespace, and comments.
// It returns a vector of tokens, which is then passed to the parser.

// 3. PARSER STRUCT
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    // Create a new parser
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // Look at the current token without consuming it
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    // Consume the current token
    fn advance(&mut self) -> Option<Token> {
        let token = self.peek()?.clone();
        self.pos += 1;
        Some(token)
    }

    // Check whether the current token matches the expected token
    fn expect(&mut self, expected: Token) -> Result<(), TelltaleParseError> {
        if self.peek() == Some(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(TelltaleParseError::UnexpectedToken)
        }
    }

    // Read an identifier
    fn expect_identifier(&mut self) -> Result<String, TelltaleParseError> {
        match self.advance() {
            Some(Token::Identifier(name)) => Ok(name),
            _ => Err(TelltaleParseError::UnexpectedToken),
        }
    }

    /// Allow an optional token
    ///
    /// If the token matches, return it, and proceed to the next token.
    /// Returns `None` if the token did not match or there are no more
    /// tokens left. Returns `Some` if the token matches and the parser's
    /// state moves forward by one token.
    fn allow(&mut self, allowed: Token) -> Option<Token> {
        if self.peek() == Some(&allowed) {
            self.advance()
        } else {
            None
        }
    }
}
//The Parser struct maintains a cursor over the token vector. peek() inspects the current token, advance() consumes it,
//and expect() validates that the next token matches the grammar.

// 4. PARSE THE COMPLETE PROGRAM

impl Parser {
    fn parse_program(&mut self) -> Result<Telltale, TelltaleParseError> {
        let mut functions = HashMap::new();

        // Parse functions until there are no more tokens
        while self.peek().is_some() {
            let function = self.parse_function()?;

            let name = function.name.clone();

            // Check for duplicate function names
            if functions.contains_key(&name) {
                return Err(TelltaleParseError::DuplicateFunction(name));
            }

            functions.insert(name, function);
        }

        // Every program must contain main
        let main = functions
            .get(MAIN_FN_NAME)
            .ok_or(TelltaleParseError::MissingMainFunction)?;

        // Main must not have parameters
        if !main.parameters.is_empty() {
            return Err(TelltaleParseError::UnexpectedToken);
        }

        Ok(Telltale {
            functions,
            globals: Vec::new(),
        })
    }
}
//parse_program() is the top-level parsing function.
//It repeatedly parses function definitions, stores them in a HashMap, checks for duplicate names,
//and verifies that the program contains a parameterless main function.

// 5. PARSE FUNCTION DECLARATIONS

impl Parser {
    fn parse_function(&mut self) -> Result<Function, TelltaleParseError> {
        self.expect(Token::Fn)?;

        // Parse optional impure designation
        let impure_designation = self.parse_effects()?;

        // Read function name
        let name = self.expect_identifier()?;

        // Parse parameters
        self.expect(Token::LeftParen)?;

        let parameters = self.parse_parameters()?;

        self.expect(Token::RightParen)?;

        // Parse function body
        self.expect(Token::LeftBrace)?;

        let body = self.parse_body()?;

        self.expect(Token::RightBrace)?;

        Ok(Function {
            name,
            impure_designation,
            parameters,
            body,
        })
    }

    // Parse impure(x, y)
    fn parse_effects(&mut self) -> Result<Option<ImpureDesignation>, TelltaleParseError> {
        if self.peek() != Some(&Token::Impure) {
            return Ok(None);
        }

        self.advance();

        self.expect(Token::LeftParen)?;

        let mut variables = Vec::new();

        if self.peek() != Some(&Token::RightParen) {
            loop {
                let name = self.expect_identifier()?;

                variables.push(Variable { name });

                if self.peek() != Some(&Token::Comma) {
                    break;
                }

                self.advance();
            }
        }

        self.expect(Token::RightParen)?;

        Ok(Some(ImpureDesignation { variables }))
    }

    // Parse function parameters
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, TelltaleParseError> {
        let mut parameters = Vec::new();

        // No parameters
        if self.peek() == Some(&Token::RightParen) {
            return Ok(parameters);
        }

        loop {
            let ref_token = self.allow(Token::Reference);
            let name = self.expect_identifier()?;

            if ref_token.is_some() {
                parameters.push(Parameter::Reference(Variable::new(name)));
            } else {
                parameters.push(Parameter::Value(Variable::new(name)));
            }

            // TODO: match right paren?
            if self.peek() != Some(&Token::Comma) {
                break;
            }

            self.advance();
        }

        Ok(parameters)
    }
}
//parse_function() combines smaller parsing functions.
//It reads the function name, optional effects, parameters, and body, then constructs the Function AST node.

// 6. PARSE FUNCTION BODY AND STATEMENTS

impl Parser {
    fn parse_body(&mut self) -> Result<Body, TelltaleParseError> {
        let mut statements = Vec::new();

        while self.peek() != Some(&Token::RightBrace) {
            // Unexpected end of input
            if self.peek().is_none() {
                return Err(TelltaleParseError::UnexpectedToken);
            }

            let statement = self.parse_statement()?;

            statements.push(statement);
        }

        Ok(Body { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, TelltaleParseError> {
        match self.peek() {
            // var x; or var x = 10;
            Some(Token::Var) => {
                self.advance();

                let name = self.expect_identifier()?;

                let variable = Variable { name };

                if self.peek() == Some(&Token::Equal) {
                    self.advance();

                    let expr = self.parse_expression(0)?;

                    self.expect(Token::Semicolon)?;

                    Ok(Statement::Declaration(variable, Some(expr)))
                } else {
                    self.expect(Token::Semicolon)?;

                    Ok(Statement::Declaration(variable, None))
                }
            }

            // return x;
            Some(Token::Return) => {
                self.advance();

                let expr = self.parse_expression(0)?;

                self.expect(Token::Semicolon)?;

                Ok(Statement::Return(expr))
            }

            // x = 10; or print(10);
            Some(Token::Identifier(_)) => {
                let name = self.expect_identifier()?;

                // Variable assignment
                if self.peek() == Some(&Token::Equal) {
                    self.advance();

                    let expr = self.parse_expression(0)?;

                    self.expect(Token::Semicolon)?;

                    Ok(Statement::Assignment(Variable { name }, expr))
                }
                // Function call
                else if self.peek() == Some(&Token::LeftParen) {
                    let call = self.parse_call_after_name(name)?;

                    self.expect(Token::Semicolon)?;

                    Ok(Statement::FunctionCall(call))
                } else {
                    Err(TelltaleParseError::UnexpectedToken)
                }
            }

            _ => Err(TelltaleParseError::UnexpectedToken),
        }
    }
}

// 7. PARSE ARITHMETIC EXPRESSIONS
// implemented using precedence climbing
impl Parser {
    fn parse_expression(&mut self, min_precedence: u8) -> Result<IntExpr, TelltaleParseError> {
        let mut left = self.parse_primary()?;

        loop {
            // Determine operator precedence
            let precedence = match self.peek() {
                Some(Token::Plus) => 1,
                Some(Token::Minus) => 1,
                Some(Token::Star) => 2,
                Some(Token::Slash) => 2,
                _ => break,
            };

            if precedence < min_precedence {
                break;
            }

            let operator = self.advance().unwrap();

            // Parse right-hand expression
            let right = self.parse_expression(precedence + 1)?;

            // Construct AST node
            left = match operator {
                Token::Plus => IntExpr::Add(Box::new(left), Box::new(right)),

                Token::Minus => IntExpr::Sub(Box::new(left), Box::new(right)),

                Token::Star => IntExpr::Mult(Box::new(left), Box::new(right)),

                Token::Slash => IntExpr::Div(Box::new(left), Box::new(right)),

                _ => unreachable!(),
            };
        }

        Ok(left)
    }

    // Parse numbers, variables and parentheses
    fn parse_primary(&mut self) -> Result<IntExpr, TelltaleParseError> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(IntExpr::Const(n)),

            Some(Token::Identifier(name)) => {
                // Function calls cannot currently be represented
                // as IntExpr in the existing core AST.
                if self.peek() == Some(&Token::LeftParen) {
                    return Err(TelltaleParseError::UnexpectedToken);
                }

                Ok(IntExpr::Var(Variable { name }))
            }

            Some(Token::LeftParen) => {
                let expr = self.parse_expression(0)?;

                self.expect(Token::RightParen)?;

                Ok(expr)
            }

            _ => Err(TelltaleParseError::UnexpectedToken),
        }
    }
}
//We use precedence climbing to parse arithmetic expressions.
//Each operator has a precedence level. When parsing an operator,
//we recursively parse its right-hand side with a higher minimum precedence.
//This ensures multiplication and division are grouped before addition and subtraction.

// 8. PARSE FUNCTION CALLS

impl Parser {
    fn parse_call_after_name(&mut self, name: String) -> Result<FunctionCall, TelltaleParseError> {
        self.expect(Token::LeftParen)?;

        let mut arguments = Vec::new();

        // Function with no arguments
        if self.peek() != Some(&Token::RightParen) {
            loop {
                // check whether this argument is a reference
                if self.allow(Token::Reference).is_some() {
                    let variable_name = self.expect_identifier()?;
                    arguments.push(Argument::Reference(Variable::new(variable_name)));
                } else {
                    let expr = self.parse_expression(0)?;
                    arguments.push(Argument::IntExpr(expr));
                }

                if self.peek() != Some(&Token::Comma) {
                    break;
                }

                self.advance();
            }
        }

        self.expect(Token::RightParen)?;

        Ok(FunctionCall {
            function_name: name,
            arguments,
        })
    }
}

// 9. PUBLIC TELLTALE PARSER

#[derive(Default)]
pub struct TelltaleParser {}

impl TelltaleParser {
    pub fn parse(&self, raw: &str) -> Result<Telltale, TelltaleParseError> {
        if raw.trim().is_empty() {
            return Err(TelltaleParseError::EmptyInput);
        }

        let tokens = tokenize(raw)?;

        let mut parser = Parser::new(tokens);

        parser.parse_program()
    }
}

// 10. TEST CASES
// 20 test cases which covers parsing, error handling, arithmatic expressions,
//function declarations,function calls and comments.

#[cfg(test)]
mod tests {
    use telltale_core::Argument;

    use super::*;

    fn parse_ok(source: &str) -> Telltale {
        TelltaleParser::default()
            .parse(source)
            .expect("Parsing should succeed")
    }

    fn parse_error(source: &str) -> TelltaleParseError {
        TelltaleParser::default()
            .parse(source)
            .expect_err("Parsing should fail")
    }

    #[test]
    fn empty_prog() {
        let program = parse_ok("fn main() {}");

        assert_eq!(program, Telltale::default());
    }

    #[test]
    fn empty_input() {
        assert_eq!(parse_error(""), TelltaleParseError::EmptyInput);
    }

    #[test]
    fn whitespace_input() {
        assert_eq!(parse_error("   \n  "), TelltaleParseError::EmptyInput);
    }

    #[test]
    fn test_variable_declaration() {
        let program = parse_ok("fn main() { var x; }");

        let main = program.functions.get("main").unwrap();

        assert!(matches!(
            &main.body.statements[0],
            Statement::Declaration(_, None)
        ));
    }

    #[test]
    fn test_variable_initialization() {
        let program = parse_ok("fn main() { var x = 10; }");

        let main = program.functions.get("main").unwrap();

        assert!(matches!(
            &main.body.statements[0],
            Statement::Declaration(_, Some(IntExpr::Const(10)))
        ));
    }

    #[test]
    fn test_assignment() {
        let program = parse_ok(
            "fn main() {
                var x = 10;
                x = 20;
            }",
        );

        let main = program.functions.get("main").unwrap();

        assert!(matches!(
            &main.body.statements[1],
            Statement::Assignment(_, IntExpr::Const(20))
        ));
    }

    #[test]
    fn test_return_statement() {
        let program = parse_ok("fn main() { return 10; }");

        let main = program.functions.get("main").unwrap();

        assert!(matches!(
            &main.body.statements[0],
            Statement::Return(IntExpr::Const(10))
        ));
    }

    #[test]
    fn test_addition() {
        let program = parse_ok("fn main() { var x = 10 + 20; }");

        let main = program.functions.get("main").unwrap();

        assert!(matches!(
            &main.body.statements[0],
            Statement::Declaration(_, Some(IntExpr::Add(_, _)))
        ));
    }

    #[test]
    fn test_operator_precedence() {
        let program = parse_ok("fn main() { var x = 10 + 20 * 3; }");

        let main = program.functions.get("main").unwrap();

        match &main.body.statements[0] {
            Statement::Declaration(_, Some(IntExpr::Add(left, right))) => {
                assert!(matches!(**left, IntExpr::Const(10)));

                assert!(matches!(**right, IntExpr::Mult(_, _)));
            }

            _ => panic!("Incorrect expression AST"),
        }
    }

    #[test]
    fn test_parentheses() {
        let program = parse_ok("fn main() { var x = (10 + 20) * 3; }");

        let main = program.functions.get("main").unwrap();

        assert!(matches!(
            &main.body.statements[0],
            Statement::Declaration(_, Some(IntExpr::Mult(_, _)))
        ));
    }

    #[test]
    fn test_function_parameters() {
        let program = parse_ok(
            "fn add(x, y) {
                return x + y;
            }

            fn main() {}",
        );

        let add = program.functions.get("add").unwrap();

        assert_eq!(add.parameters.len(), 2);
    }

    #[test]
    fn test_function_call() {
        let program = parse_ok(
            "fn main() {
                print(10);
            }",
        );

        let main = program
            .functions
            .get("main")
            .expect("main function was not parsed");
        assert!(
            main.body.statements.len() == 1,
            "main does not have exactly one statement"
        );
        match main.body.statements.get(0) {
            Some(Statement::FunctionCall(function_call)) => {
                assert_eq!(
                    function_call,
                    &FunctionCall {
                        function_name: "print".to_string(),
                        arguments: vec![Argument::IntExpr(IntExpr::Const(10))]
                    },
                    "function call was parsed incorrectly"
                );
            }
            None | Some(_) => panic!("statement is not a function call"),
        };
    }

    #[test]
    fn test_function_call_arguments() {
        let program = parse_ok(
            "fn main() {
                add(10, 20);
            }",
        );

        let main = program.functions.get("main").unwrap();

        match &main.body.statements[0] {
            Statement::FunctionCall(call) => {
                assert_eq!(call.function_name, "add");
                assert_eq!(call.arguments.len(), 2);
            }

            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_impure_function() {
        let program = parse_ok(
            "fn impure(x) update(y) {
            }

            fn main() {}",
        );

        let update = program.functions.get("update").unwrap();

        assert!(update.impure_designation.is_some());

        assert_eq!(
            update.impure_designation.as_ref().unwrap().variables.len(),
            1
        );
    }

    #[test]
    fn test_missing_main() {
        assert_eq!(
            parse_error("fn add(x, y) {}"),
            TelltaleParseError::MissingMainFunction
        );
    }

    #[test]
    fn test_main_with_parameters() {
        assert_eq!(
            parse_error("fn main(x) {}"),
            TelltaleParseError::UnexpectedToken
        );
    }

    #[test]
    fn test_duplicate_function() {
        assert_eq!(
            parse_error(
                "fn main() {}
                 fn main() {}"
            ),
            TelltaleParseError::DuplicateFunction("main".to_string())
        );
    }

    #[test]
    fn test_missing_semicolon() {
        assert_eq!(
            parse_error("fn main() { var x = 10 }"),
            TelltaleParseError::UnexpectedToken
        );
    }

    #[test]
    fn test_missing_brace() {
        assert_eq!(
            parse_error("fn main() {"),
            TelltaleParseError::UnexpectedToken
        );
    }

    #[test]
    fn test_invalid_character() {
        assert_eq!(
            parse_error("fn main() { @ }"),
            TelltaleParseError::UnexpectedCharacter('@')
        );
    }

    #[test]
    fn test_comments() {
        let program = parse_ok(
            "fn main() {
                // This is a comment
                var x = 10;
            }",
        );

        let main = program.functions.get("main").unwrap();

        assert_eq!(main.body.statements.len(), 1);
    }

    #[test]
    fn test_function_values() {
        let program = parse_ok(
            "
            fn add(x, y) {
                return x + y;
            }

            fn main() {
                var x = 10;
                add(x, x);
            }",
        );

        let main = program
            .functions
            .get("main")
            .expect("main function was not parsed");
        let add = program
            .functions
            .get("add")
            .expect("add function was not parsed");
        match main.body.statements.get(1) {
            Some(Statement::FunctionCall(function_call)) => {
                assert_eq!(
                    function_call,
                    &FunctionCall {
                        function_name: "add".to_string(),
                        arguments: vec![
                            Argument::IntExpr(IntExpr::Var(Variable::new("x"))),
                            Argument::IntExpr(IntExpr::Var(Variable::new("x")))
                        ]
                    },
                    "add function call was parsed incorrectly"
                );
            }
            None | Some(_) => panic!("main body was parsed incorrectly"),
        };
        assert_eq!(
            add.parameters,
            vec![
                Parameter::Value(Variable::new("x")),
                Parameter::Value(Variable::new("y"))
            ],
            "add parameters are incorrect"
        );
    }

    #[test]
    fn test_function_references() {
        let program = parse_ok(
            "
            fn add(&x, &y) {
                return x + y;
            }

            fn main() {
                var x = 10;
                var y = 10;
                add(&x, &y);
            }",
        );

        let main = program
            .functions
            .get("main")
            .expect("main function was not parsed");
        let add = program
            .functions
            .get("add")
            .expect("add function was not parsed");
        match main.body.statements.get(2) {
            Some(Statement::FunctionCall(function_call)) => {
                assert_eq!(
                    function_call,
                    &FunctionCall {
                        function_name: "add".to_string(),
                        arguments: vec![
                            Argument::Reference(Variable::new("x")),
                            Argument::Reference(Variable::new("y"))
                        ]
                    },
                    "add function call was parsed incorrectly"
                );
            }
            None | Some(_) => panic!("main body was parsed incorrectly"),
        };
        assert_eq!(
            add.parameters,
            vec![
                Parameter::Reference(Variable::new("x")),
                Parameter::Reference(Variable::new("y"))
            ],
            "add parameters are incorrect"
        );
    }
}
