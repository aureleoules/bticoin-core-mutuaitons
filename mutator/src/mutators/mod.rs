use fancy_regex::Regex;

pub mod execution_flow;
pub mod operator;
pub mod std_algorithm;

pub struct MutatorContext {
    pub file: String,
    pub line: usize,
    pub line_content: String,
}

pub trait Mutator {
    fn mutate(&self, ctx: &MutatorContext) -> Vec<String>;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

struct SimpleMutation {
    pub from: Regex,
    pub to: Vec<&'static str>,
}

fn simple_mutate(line: &str, patterns: &Vec<SimpleMutation>) -> Vec<String> {
    let mut mutations = vec![];

    for simple_mutation in patterns {
        let matches = simple_mutation.from.find_iter(line);

        let string_litterals = find_string_literals(line);

        for m in matches.flatten() {
            if string_litterals
                .iter()
                .any(|(start, end)| m.start() >= *start && m.end() <= *end)
            {
                continue;
            }

            for to in &simple_mutation.to {
                let mut mutated_line = line.to_string();
                mutated_line.replace_range(m.start()..m.end(), to);
                mutations.push(mutated_line);
            }
        }
    }

    mutations
}

fn find_string_literals(line: &str) -> Vec<(usize, usize)> {
    let mut result = vec![];
    let mut in_string = false;
    let mut start = 0;
    for (i, c) in line.chars().enumerate() {
        if in_string {
            if c == '"' {
                in_string = false;
                result.push((start, i));
            }
        } else if c == '"' {
            in_string = true;
            start = i;
        }
    }
    result
}
