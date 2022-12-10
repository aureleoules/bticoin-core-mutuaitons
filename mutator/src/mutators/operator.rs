use fancy_regex::Regex;

use super::{simple_mutate, Mutator, MutatorContext, SimpleMutation};

pub struct OperatorMutator {
    patterns: Vec<SimpleMutation>,
}

impl Mutator for OperatorMutator {
    fn name(&self) -> &'static str {
        "OperatorMutator"
    }

    fn description(&self) -> &'static str {
        "Mutates operators such as == to !=, < to >=, etc."
    }

    fn mutate(&self, ctx: &MutatorContext) -> Vec<String> {
        simple_mutate(&ctx.line_content, &self.patterns)
    }
}

impl Default for OperatorMutator {
    fn default() -> Self {
        Self {
            patterns: vec![
                SimpleMutation {
                    from: Regex::new(r"==").unwrap(),
                    to: vec!["!=", "<", ">", "<=", ">="],
                },
                SimpleMutation {
                    from: Regex::new(r"!=").unwrap(),
                    to: vec!["==", "<", ">", "<=", ">="],
                },
                SimpleMutation {
                    from: Regex::new(r"<").unwrap(),
                    to: vec!["==", "!=", ">", "<=", ">="],
                },
                SimpleMutation {
                    from: Regex::new(r">").unwrap(),
                    to: vec!["==", "!=", "<", "<=", ">="],
                },
                SimpleMutation {
                    from: Regex::new(r"<=").unwrap(),
                    to: vec!["==", "!=", "<", ">", ">="],
                },
                SimpleMutation {
                    from: Regex::new(r">=").unwrap(),
                    to: vec!["==", "!=", "<", ">", "<="],
                },
            ],
        }
    }
}

pub struct BoolOperatorMutator {
    patterns: Vec<SimpleMutation>,
}

impl Mutator for BoolOperatorMutator {
    fn name(&self) -> &'static str {
        "BoolOperatorMutator"
    }

    fn description(&self) -> &'static str {
        "Mutates boolean operators such as && to ||, || to &&, etc."
    }

    fn mutate(&self, ctx: &MutatorContext) -> Vec<String> {
        simple_mutate(&ctx.line_content, &self.patterns)
    }
}

impl Default for BoolOperatorMutator {
    fn default() -> Self {
        Self {
            patterns: vec![
                SimpleMutation {
                    from: Regex::new(r"&&").unwrap(),
                    to: vec!["||"],
                },
                SimpleMutation {
                    from: Regex::new(r"\|\|").unwrap(),
                    to: vec!["&&"],
                },
                SimpleMutation {
                    from: Regex::new(r"false").unwrap(),
                    to: vec!["true"],
                },
                SimpleMutation {
                    from: Regex::new(r"true").unwrap(),
                    to: vec!["false"],
                },
                SimpleMutation {
                    from: Regex::new(r"!").unwrap(),
                    to: vec![""],
                },
            ],
        }
    }
}

pub struct BoolAritmeticMutator {
    patterns: Vec<SimpleMutation>,
}

impl Mutator for BoolAritmeticMutator {
    fn name(&self) -> &'static str {
        "BoolAritmeticMutator"
    }

    fn description(&self) -> &'static str {
        "Mutates boolean arithmetic operators such as & to |, | to &, etc."
    }

    fn mutate(&self, ctx: &MutatorContext) -> Vec<String> {
        simple_mutate(&ctx.line_content, &self.patterns)
    }
}

impl Default for BoolAritmeticMutator {
    fn default() -> Self {
        Self {
            patterns: vec![
                SimpleMutation {
                    from: Regex::new(r" & ").unwrap(),
                    to: vec![" | ", " ^ ", " << ", " >> "],
                },
                SimpleMutation {
                    from: Regex::new(r" \| ").unwrap(),
                    to: vec![" & ", " ^ ", " << ", " >> "],
                },
                SimpleMutation {
                    from: Regex::new(r" ^ ").unwrap(),
                    to: vec![" & ", " | ", " << ", " >> "],
                },
                SimpleMutation {
                    from: Regex::new(r" << ").unwrap(),
                    to: vec![" & ", " | ", " ^ ", " >> "],
                },
                SimpleMutation {
                    from: Regex::new(r" >> ").unwrap(),
                    to: vec![" & ", " | ", " ^ ", " << "],
                },
            ],
        }
    }
}

pub struct IncDecMutator {
    patterns: Vec<SimpleMutation>,
}

impl Mutator for IncDecMutator {
    fn name(&self) -> &'static str {
        "IncDecMutator"
    }

    fn description(&self) -> &'static str {
        "Mutates increment and decrement operators such as ++ to --, -- to ++, etc."
    }

    fn mutate(&self, ctx: &MutatorContext) -> Vec<String> {
        simple_mutate(&ctx.line_content, &self.patterns)
    }
}

impl Default for IncDecMutator {
    fn default() -> Self {
        Self {
            patterns: vec![
                SimpleMutation {
                    from: Regex::new(r"\+\+").unwrap(),
                    to: vec!["--"],
                },
                SimpleMutation {
                    from: Regex::new(r"--").unwrap(),
                    to: vec!["++"],
                },
            ],
        }
    }
}
