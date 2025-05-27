use crate::analyzer::{AnalyzedProgram, TestInfo};
use crate::parser::AstNode;
use std::collections::HashMap;
use regex::Regex;
use std::fmt;

/// Result of running a single test
#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
    pub error_message: Option<String>,
}

impl fmt::Display for TestResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.passed {
            write!(f, "✓ {}", self.test_name)
        } else {
            writeln!(f, "✗ {}", self.test_name)?;
            writeln!(f, "  Expected: {}", self.expected)?;
            writeln!(f, "  Actual:   {}", self.actual)?;
            if let Some(error) = &self.error_message {
                writeln!(f, "  Error:    {}", error)?;
            }
            Ok(())
        }
    }
}

/// Summary of all test results
#[derive(Debug, Clone)]
pub struct TestSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub results: Vec<TestResult>,
}

impl fmt::Display for TestSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "\nTest Results:")?;
        for result in &self.results {
            writeln!(f, "{}", result)?;
        }
        writeln!(f, "\nSummary: {} passed, {} failed, {} total", 
                 self.passed, self.failed, self.total)?;
        if self.failed > 0 {
            writeln!(f, "FAILED")?;
        } else {
            writeln!(f, "PASSED")?;
        }
        Ok(())
    }
}

/// Test execution engine that runs tests against analyzed programs
pub struct TestRunner {
    verbose: bool,
}

impl TestRunner {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Run all tests in the analyzed program
    pub fn run_tests(&self, analyzed: &AnalyzedProgram) -> TestSummary {
        let tests = &analyzed.tests;
        let total = tests.len();

        // Debug output
        println!("Found {} test(s) in analyzed program", total);
        if self.verbose && total == 0 {
            println!("No tests found. Check if @test annotations are being parsed correctly.");
        }

        if self.verbose {
            println!("Running {} test(s)...", total);
        }

        let mut results = Vec::new();
        let mut passed = 0;
        let mut failed = 0;

        for test in tests {
            if self.verbose {
                println!("Running test: {}", test.name);
            }

            let result = self.run_single_test(test, analyzed);
            
            if result.passed {
                passed += 1;
            } else {
                failed += 1;
            }

            results.push(result);
        }

        TestSummary {
            total,
            passed,
            failed,
            results,
        }
    }

    /// Run a single test case
    fn run_single_test(&self, test: &TestInfo, analyzed: &AnalyzedProgram) -> TestResult {
        // Evaluate the expected output
        let expected_value = match self.evaluate_ast_node(&test.expected_output, analyzed) {
            Ok(value) => value,
            Err(error) => {
                return TestResult {
                    test_name: test.name.clone(),
                    passed: false,
                    expected: "".to_string(),
                    actual: "".to_string(),
                    error_message: Some(format!("Failed to evaluate expected output: {}", error)),
                };
            }
        };

        // Evaluate the actual value by executing the expect expression against the program's symbol table
        let actual_value = match self.evaluate_ast_node(&test.expect_expression, analyzed) {
            Ok(value) => value,
            Err(error) => {
                return TestResult {
                    test_name: test.name.clone(),
                    passed: false,
                    expected: self.value_to_string(&expected_value),
                    actual: "".to_string(),
                    error_message: Some(format!("Failed to evaluate expect expression: {}", error)),
                };
            }
        };

        // Compare expected vs actual using deep value comparison
        let passed = if test.is_regex {
            match Regex::new(&self.value_to_string(&expected_value)) {
                Ok(regex) => regex.is_match(&self.value_to_string(&actual_value)),
                Err(error) => {
                    return TestResult {
                        test_name: test.name.clone(),
                        passed: false,
                        expected: self.value_to_string(&expected_value),
                        actual: self.value_to_string(&actual_value),
                        error_message: Some(format!("Invalid regex pattern: {}", error)),
                    };
                }
            }
        } else {
            self.values_equal(&expected_value, &actual_value)
        };

        TestResult {
            test_name: test.name.clone(),
            passed,
            expected: self.value_to_string(&expected_value),
            actual: self.value_to_string(&actual_value),
            error_message: None,
        }
    }

    /// Create a test environment with input values set as variables
    fn create_test_environment(&self, inputs: &HashMap<String, AstNode>, analyzed: &AnalyzedProgram) -> TestEnvironment {
        let mut variables = HashMap::new();
        
        for (name, ast_node) in inputs {
            match self.evaluate_ast_node(ast_node, analyzed) {
                Ok(value) => {
                    variables.insert(name.clone(), value);
                }
                Err(error) => {
                    if self.verbose {
                        println!("Warning: Failed to evaluate input '{}': {}", name, error);
                    }
                    variables.insert(name.clone(), TestValue::Null);
                }
            }
        }

        TestEnvironment { variables }
    }

    /// Evaluate an AST node to produce a test value
    fn evaluate_ast_node(&self, node: &AstNode, analyzed: &AnalyzedProgram) -> Result<TestValue, String> {
        match node {
            AstNode::String(s) => Ok(TestValue::String(s.clone())),
            AstNode::Number(n) => Ok(TestValue::Number(*n)),
            AstNode::Boolean(b) => Ok(TestValue::Boolean(*b)),
            AstNode::Null => Ok(TestValue::Null),
            AstNode::Array { elements, .. } => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_ast_node(element, analyzed)?);
                }
                Ok(TestValue::Array(values))
            }
            AstNode::Object { fields, .. } => {
                let mut object = HashMap::new();
                for (key, value) in fields {
                    object.insert(key.clone(), self.evaluate_ast_node(value, analyzed)?);
                }
                Ok(TestValue::Object(object))
            }
            AstNode::Identifier(name) => {
                // Look up the identifier in the program's symbol table
                if let Some(symbol) = analyzed.symbol_table.get_symbol(name) {
                    if let Some(ast_node) = &symbol.ast_node {
                        // Recursively evaluate the symbol's AST node
                        self.evaluate_ast_node(ast_node, analyzed)
                    } else {
                        Err(format!("Symbol '{}' has no AST node", name))
                    }
                } else {
                    Err(format!("Undefined symbol '{}'", name))
                }
            }
            AstNode::MemberAccess { object, property, .. } => {
                // Evaluate the object first
                let object_value = self.evaluate_ast_node(object, analyzed)?;
                
                // Extract the property from the object
                match object_value {
                    TestValue::Object(ref obj) => {
                        if let Some(property_value) = obj.get(property) {
                            Ok(property_value.clone())
                        } else {
                            Err(format!("Property '{}' not found in object", property))
                        }
                    }
                    _ => {
                        Err(format!("Cannot access property '{}' on non-object value", property))
                    }
                }
            }
            _ => {
                Err(format!("Cannot evaluate AST node type: {:?}", std::mem::discriminant(node)))
            }
        }
    }

    /// Convert a test value to its string representation
    fn value_to_string(&self, value: &TestValue) -> String {
        match value {
            TestValue::Null => "null".to_string(),
            TestValue::Boolean(b) => b.to_string(),
            TestValue::Number(n) => n.to_string(),
            TestValue::String(s) => s.clone(),
            TestValue::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| self.value_to_string(v)).collect();
                format!("[{}]", elements.join(", "))
            }
            TestValue::Object(obj) => {
                let pairs: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_string(v)))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
        }
    }

    /// Compare two test values for equality, handling objects with different field ordering
    fn values_equal(&self, a: &TestValue, b: &TestValue) -> bool {
        use TestValue::*;

        match (a, b) {
            (Null, Null) => true,
            (Boolean(ba), Boolean(bb)) => ba == bb,
            (Number(na), Number(nb)) => (na - nb).abs() < f64::EPSILON,
            (String(sa), String(sb)) => sa == sb,
            (Array(ae), Array(be)) => {
                if ae.len() != be.len() {
                    return false;
                }
                ae.iter().zip(be).all(|(av, bv)| self.values_equal(av, bv))
            }
            (Object(ao), Object(bo)) => {
                // Compare objects field by field, ignoring order
                if ao.len() != bo.len() {
                    return false;
                }
                
                // Check that all fields in ao exist in bo with equal values
                for (key, value_a) in ao {
                    match bo.get(key) {
                        Some(value_b) => {
                            if !self.values_equal(value_a, value_b) {
                                return false;
                            }
                        },
                        None => return false,
                    }
                }
                true
            }
            _ => false,
        }
    }


}

/// Test execution environment containing input variables
#[derive(Debug, Clone)]
pub struct TestEnvironment {
    pub variables: HashMap<String, TestValue>,
}

/// Simplified value type for test execution
#[derive(Debug, Clone, PartialEq)]
pub enum TestValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<TestValue>),
    Object(HashMap<String, TestValue>),
}

impl fmt::Display for TestValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestValue::Null => write!(f, "null"),
            TestValue::Boolean(b) => write!(f, "{}", b),
            TestValue::Number(n) => write!(f, "{}", n),
            TestValue::String(s) => write!(f, "{}", s),
            TestValue::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            TestValue::Object(obj) => {
                let pairs: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
        }
    }
}

/// Factory function for creating test runners
pub fn create_test_runner(verbose: bool) -> TestRunner {
    TestRunner::new(verbose)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::SemanticAnalyzer;
    use crate::parser::{Parser, AstNode};
    use crate::lexer::Lexer;

    fn create_test_program(source: &str) -> AnalyzedProgram {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast).unwrap()
    }

    #[test]
    fn test_runner_creation() {
        let runner = TestRunner::new(true);
        assert!(runner.verbose);
        
        let runner = TestRunner::new(false);
        assert!(!runner.verbose);
    }

    #[test]
    fn test_value_to_string() {
        let runner = TestRunner::new(false);
        
        assert_eq!(runner.value_to_string(&TestValue::Null), "null");
        assert_eq!(runner.value_to_string(&TestValue::Boolean(true)), "true");
        assert_eq!(runner.value_to_string(&TestValue::Number(42.0)), "42");
        assert_eq!(runner.value_to_string(&TestValue::String("hello".to_string())), "hello");
    }

    #[test]
    fn test_evaluate_ast_node() {
        let runner = TestRunner::new(false);
        let program = create_test_program("x = 5");
        
        let string_node = AstNode::String("test".to_string());
        let result = runner.evaluate_ast_node(&string_node, &program).unwrap();
        assert_eq!(result, TestValue::String("test".to_string()));
        
        let number_node = AstNode::Number(42.0);
        let result = runner.evaluate_ast_node(&number_node, &program).unwrap();
        assert_eq!(result, TestValue::Number(42.0));
        
        let bool_node = AstNode::Boolean(true);
        let result = runner.evaluate_ast_node(&bool_node, &program).unwrap();
        assert_eq!(result, TestValue::Boolean(true));
    }

    #[test]
    fn test_create_test_environment() {
        let runner = TestRunner::new(false);
        let program = create_test_program("x = 5");
        
        let mut inputs = HashMap::new();
        inputs.insert("test_var".to_string(), AstNode::String("hello".to_string()));
        inputs.insert("test_num".to_string(), AstNode::Number(42.0));
        
        let env = runner.create_test_environment(&inputs, &program);
        
        assert_eq!(env.variables.len(), 2);
        assert_eq!(env.variables.get("test_var"), Some(&TestValue::String("hello".to_string())));
        assert_eq!(env.variables.get("test_num"), Some(&TestValue::Number(42.0)));
    }

    #[test]
    fn test_test_summary_display() {
        let results = vec![
            TestResult {
                test_name: "test1".to_string(),
                passed: true,
                expected: "hello".to_string(),
                actual: "hello".to_string(),
                error_message: None,
            },
            TestResult {
                test_name: "test2".to_string(),
                passed: false,
                expected: "world".to_string(),
                actual: "foo".to_string(),
                error_message: None,
            },
        ];

        let summary = TestSummary {
            total: 2,
            passed: 1,
            failed: 1,
            results,
        };

        let output = summary.to_string();
        assert!(output.contains("test1"));
        assert!(output.contains("test2"));
        assert!(output.contains("1 passed, 1 failed, 2 total"));
    }
}