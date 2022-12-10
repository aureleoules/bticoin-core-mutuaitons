use fancy_regex::Regex;

use super::{simple_mutate, Mutator, MutatorContext, SimpleMutation};

pub struct ExecutionFlowMutator {
    patterns: Vec<SimpleMutation>,
}

impl Default for ExecutionFlowMutator {
    fn default() -> Self {
        Self {
            patterns: vec![
                SimpleMutation {
                    from: Regex::new(r"return;").unwrap(),
                    to: vec!["break;", "continue;"],
                },
                SimpleMutation {
                    from: Regex::new(r"break").unwrap(),
                    to: vec!["return", "continue"],
                },
                SimpleMutation {
                    from: Regex::new(r"continue").unwrap(),
                    to: vec!["return", "break"],
                },
            ],
        }
    }
}

impl Mutator for ExecutionFlowMutator {
    fn name(&self) -> &'static str {
        "ExecutionFlowMutator"
    }

    fn description(&self) -> &'static str {
        "Mutates execution flow operators such as return to break, continue, etc."
    }

    fn mutate(&self, ctx: &MutatorContext) -> Vec<String> {
        simple_mutate(&ctx.line_content, &self.patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mutators::MutatorContext;

    struct TestCase {
        line: &'static str,
        expected: Vec<&'static str>,
    }

    #[test]
    fn test_execution_flow_mutator() {
        let mutator = ExecutionFlowMutator::default();

        let tests = vec![
            TestCase {
                line: "return",
                expected: vec!["break", "continue"],
            },
            TestCase {
                line: "break",
                expected: vec!["return", "continue"],
            },
            TestCase {
                line: "continue",
                expected: vec!["return", "break"],
            },
        ];

        for test in tests {
            let ctx = MutatorContext {
                line_content: test.line.to_string(),
                file: "".to_string(),
                line: 0,
            };
            let actual = mutator.mutate(&ctx);
            assert_eq!(actual, test.expected);
        }
    }
}
