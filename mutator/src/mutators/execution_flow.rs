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
                    from: Regex::new(r"return\s+(.*)\s*").unwrap(),
                    to: vec!["break;", "continue;"],
                },
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
                SimpleMutation {
                    from: Regex::new(r"at(\(.\))").unwrap(),
                    to: vec!["at(0)"]
                },
                SimpleMutation {
                    from: Regex::new(r"JSONRPCError(\(.*\))").unwrap(),
                    to: vec!["JSONRPCError(-123123, \"abc123\")"]
                }
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
                line: "return wallet.GetLastBlockHash();",
                expected: vec!["break;", "continue;"],
            },
            TestCase {
                line: "return;",
                expected: vec!["break;", "continue;"],
            },
            TestCase {
                line: "return true;",
                expected: vec!["break;", "continue;"],
            },
            TestCase {
                line: "break",
                expected: vec!["return", "continue"],
            },
            TestCase {
                line: "continue",
                expected: vec!["return", "break"],
            },
            TestCase {
                line: "tx.vin.at(i).scriptSig",
                expected: vec!["tx.vin.at(0).scriptSig"]
            },
            TestCase {
                line: "txdata.m_spent_outputs.at(i);",
                expected: vec!["txdata.m_spent_outputs.at(0);"]
            },
            TestCase {
                line: "throw JSONRPCError(RPC_WALLET_ERROR, \"Error: Private keys are disabled for this wallet\");",
                expected: vec!["throw JSONRPCError(-123123, \"abc123\");"]
            }
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
