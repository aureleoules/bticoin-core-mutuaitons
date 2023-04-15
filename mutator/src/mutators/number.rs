use fancy_regex::Regex;

use super::{number_mutate, Mutator, MutatorContext};

pub struct NumberMutator {
    patterns: Vec<Regex>,
}

impl Mutator for NumberMutator {
    fn name(&self) -> &'static str {
        "NumberMutator"
    }

    fn description(&self) -> &'static str {
        "Mutates number type stuff"
    }

    fn mutate(&self, ctx: &MutatorContext) -> Vec<String> {
        number_mutate(&ctx.line_content, &self.patterns)
    }
}

impl Default for NumberMutator {
    fn default() -> Self {
        let patterns = vec![
            Regex::new(r"\b((?:int|uint\d+_t|long|unsigned long|long long|unsigned long long|CAmount))\s+([a-zA-Z_]\w*)\s*=\s*(-?\d+)").unwrap()
            //Regex::new(r"\b((?:int|uint\d+_t|long|unsigned long|long long|unsigned long long|CAmount))\s+([a-zA-Z_]\w*)\s*(?:=|\{)\s*(-?\d+)\s*(?:\}|\;)").unwrap()
        ];

        Self { patterns }
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
        let mutator = NumberMutator::default();

        let tests = vec![
            TestCase {
                line: "CAmount bla = 1",
                expected: vec!["CAmount bla = 0", "CAmount bla = 2"],
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
