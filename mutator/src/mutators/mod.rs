use fancy_regex::Regex;

pub mod execution_flow;
pub mod operator;
pub mod std_algorithm;
pub mod number;

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

fn number_mutate(line: &str, patterns: &Vec<Regex>) -> Vec<String> {
    let mut mutations = vec![];

    for pattern in patterns {
        if let Some(captures) = pattern.captures(line).unwrap() {
            let type_str = captures.get(1).unwrap().as_str();
            let name_str = captures.get(2).unwrap().as_str();
            let value_str = captures.get(3).unwrap().as_str();

            if ["int64_t", "long", "long long", "CAmount"].contains(&type_str) {
                let value: i64 = value_str.parse::<i64>().unwrap();
                let percentages = vec![0i64, 2i64];
                for percen in percentages {
                    mutations.push(format!("{} {} = {}", type_str, name_str, value * percen));
                }
            } else if ["uint64_t", "unsigned long", "unsigned long long"].contains(&type_str) {
                let value: u64 = value_str.parse::<u64>().unwrap();
                let percentages = vec![0u64, 2u64];
                for percen in percentages {
                    mutations.push(format!("{} {} = {}", type_str, name_str, value * percen));
                }
            } else if ["uint32_t", "unsigned int"].contains(&type_str) {
                let value: u32 = value_str.parse::<u32>().unwrap();
                let percentages = vec![0u32, 2u32];
                for percen in percentages {
                    mutations.push(format!("{} {} = {}", type_str, name_str, value * percen));
                }
            } else if ["int32_t", "int"].contains(&type_str) {
                let value: i32 = value_str.parse::<i32>().unwrap();
                let percentages = vec![0i32, 2i32];
                for percen in percentages {
                    mutations.push(format!("{} {} = {}", type_str, name_str, value * percen));
                }
            }
        }
    }

    mutations
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
