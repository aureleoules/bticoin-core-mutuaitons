use fancy_regex::Regex;

use super::{simple_mutate, Mutator, MutatorContext, SimpleMutation};

pub struct StdAlgorithmMutator {
    patterns: Vec<SimpleMutation>,
}

impl Mutator for StdAlgorithmMutator {
    fn name(&self) -> &'static str {
        "StdAlgorithmMutator"
    }

    fn description(&self) -> &'static str {
        "Mutates std::algorithm functions such as std::sort to std::stable_sort, std::min to std::max, etc."
    }

    fn mutate(&self, ctx: &MutatorContext) -> Vec<String> {
        simple_mutate(&ctx.line_content, &self.patterns)
    }
}

impl Default for StdAlgorithmMutator {
    fn default() -> Self {
        let patterns = vec![
            SimpleMutation {
                from: Regex::new(r"std::sort").unwrap(),
                to: vec!["std::stable_sort"],
            },
            SimpleMutation {
                from: Regex::new(r"std::min").unwrap(),
                to: vec!["std::max"],
            },
            SimpleMutation {
                from: Regex::new(r"std::max").unwrap(),
                to: vec!["std::min"],
            },
            SimpleMutation {
                from: Regex::new(r"std::min_element").unwrap(),
                to: vec!["std::max_element"],
            },
            SimpleMutation {
                from: Regex::new(r"std::max_element").unwrap(),
                to: vec!["std::min_element"],
            },
            SimpleMutation {
                from: Regex::new(r"std::all_of").unwrap(),
                to: vec!["std::any_of", "std::none_of"],
            },
            SimpleMutation {
                from: Regex::new(r"std::any_of").unwrap(),
                to: vec!["std::all_of", "std::none_of"],
            },
            SimpleMutation {
                from: Regex::new(r"std::none_of").unwrap(),
                to: vec!["std::all_of", "std::any_of"],
            },
            SimpleMutation {
                from: Regex::new(r"std::accumulate").unwrap(),
                to: vec!["std::inner_product"],
            },
            SimpleMutation {
                from: Regex::new(r"std::inner_product").unwrap(),
                to: vec!["std::accumulate"],
            },
            SimpleMutation {
                from: Regex::new(r"std::begin").unwrap(),
                to: vec!["std::end"],
            },
            SimpleMutation {
                from: Regex::new(r"std::end").unwrap(),
                to: vec!["std::begin"],
            },
            SimpleMutation {
                from: Regex::new(r"std::cbegin").unwrap(),
                to: vec!["std::cend"],
            },
            SimpleMutation {
                from: Regex::new(r"std::cend").unwrap(),
                to: vec!["std::cbegin"],
            },
            SimpleMutation {
                from: Regex::new(r"std::rbegin").unwrap(),
                to: vec!["std::rend"],
            },
            SimpleMutation {
                from: Regex::new(r"std::rend").unwrap(),
                to: vec!["std::rbegin"],
            },
            SimpleMutation {
                from: Regex::new(r"std::crbegin").unwrap(),
                to: vec!["std::crend"],
            },
            SimpleMutation {
                from: Regex::new(r"std::crend").unwrap(),
                to: vec!["std::crbegin"],
            },
            SimpleMutation {
                from: Regex::new(r"std::next").unwrap(),
                to: vec!["std::prev"],
            },
            SimpleMutation {
                from: Regex::new(r"std::prev").unwrap(),
                to: vec!["std::next"],
            },
            SimpleMutation {
                from: Regex::new(r"std::front_inserter").unwrap(),
                to: vec!["std::back_inserter"],
            },
            SimpleMutation {
                from: Regex::new(r"std::back_inserter").unwrap(),
                to: vec!["std::front_inserter"],
            },
        ];

        Self { patterns }
    }
}
