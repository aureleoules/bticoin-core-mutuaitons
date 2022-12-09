use common::{Mutation, MutationStatus};
use fancy_regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref MUTATORS: Vec<(Regex, String)> = get_mutators();
}

fn get_mutators() -> Vec<(Regex, String)> {
    vec![
        (r"at\((.*)\)".to_string(), "at(0)".to_string()),
        // (r"CAmount\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"CAmount \1 = RANDOM_INT".to_string()),
        // (r"CAmount\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"CAmount \1 = LESS".to_string()),
        // (r"CAmount{+(\w+)}".to_string(), r"CAmount{RANDOM_INT}".to_string()),
        // (r"int\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"int \1 = RANDOM_INT".to_string()),
        // (r"int\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"int \1 = LESS".to_string()),
        // (r"int\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"int \1 = GREATER".to_string()),
        // (r"int32_t\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"int32_t \1 = LESS".to_string()),
        // (r"int32_t\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"int32_t \1 = GREATER".to_string()),
        // (r"int64_t\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"int64_t \1 = RANDOM_INT".to_string()),
        // (r"int64_t\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"int64_t \1 = GREATER".to_string()),
        // (r"int64_t\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"int64_t \1 = LESS".to_string()),
        // (r"std::chrono::seconds (\w+){+(\w+)}".to_string(), r"std::chrono::seconds \1{RANDOM_INT}".to_string()),
        // (r"int32_t\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"int32_t \1 = RANDOM_INT".to_string()),
        // (r"const\s+size_t\s+(\w+)\s*=\s*([0-9]+)".to_string(), r"const size_t \1 = LESS".to_string()),
        // (r"std::chrono::seconds (\w+){+(\w+)}".to_string(), r"std::chrono::seconds \1{RANDOM_INT}".to_string()),
        // (r"std::chrono::seconds (\w+)\s*=\s*(.*)".to_string(), r"std::chrono::seconds \1 = RANDOM_INT;".to_string()),
        (r" break".to_string(), " continue".to_string()),
        (r" continue".to_string(), " break".to_string()),
        (r"std::all_of".to_string(), "std::any_of".to_string()),
        (r"std::any_of".to_string(), "std::all_of".to_string()),
        (r"std::min".to_string(), "std::max".to_string()),
        (r"std::max".to_string(), "std::min".to_string()),
        (r"std::begin".to_string(), "std::end".to_string()),
        (r"std::end".to_string(), "std::begin".to_string()),
        (r"true".to_string(), "false".to_string()),
        (r"false".to_string(), "true".to_string()),
        (r" > ".to_string(), " < ".to_string()),
        (r" < ".to_string(), " > ".to_string()),
        (r" >= ".to_string(), " > ".to_string()),
        (r" >= ".to_string(), " <= ".to_string()),
        (r" <= ".to_string(), " > ".to_string()),
        (r" <= ".to_string(), " >= ".to_string()),
        (r" == ".to_string(), " != ".to_string()),
        (r" != ".to_string(), " == ".to_string()),
    ]
    .into_iter()
    .map(|(a, b)| (Regex::new(&a).unwrap(), b))
    .collect()
}

pub fn create_mutations(lines: &Vec<&str>, muts: &mut Vec<(usize, String)>) {
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("assert") {
            continue;
        }

        for (regex, replacement) in MUTATORS.iter() {
            // Check if match
            if regex.is_match(line).unwrap() {
                let new_line = regex.replace(line, replacement).to_string();
                muts.push((i, new_line));
            }
        }
    }
}

pub fn create_mutations_from_files(files: &Vec<String>) -> Vec<Mutation> {
    let mut mutations = Vec::new();
    for file in files {
        println!("File: {}", file);

        let content = std::fs::read_to_string(&file).unwrap();
        let lines = content.split('\n').collect::<Vec<&str>>();
        let mut muts: Vec<(usize, String)> = vec![]; // (line, mutation)

        create_mutations(&lines, &mut muts);

        println!("{} mutations found", muts.len());

        // Generate Git diff patch
        for (line, mutation) in muts {
            let mut lines_copy = lines.clone();
            lines_copy[line] = mutation.as_str();
            let joined = lines_copy.join("\n");
            let patch = diffy::create_patch(&content, &joined);

            let md5 = md5::compute(patch.to_bytes()).to_vec();
            let md5 = hex::encode(md5);
            let m = Mutation {
                id: 0,
                file: Some(file.clone()),
                patch_md5: Some(md5),
                line: Some(line as i64),
                patch: Some(patch.to_string()),
                branch: Some("master".to_string()),
                pr_number: None,
                status: Some(MutationStatus::Pending.to_string()),
                start_time: None,
                end_time: None,
                stderr: None,
                stdout: None,
            };

            mutations.push(m);
        }
    }

    mutations
}
pub async fn send_mutations(
    server: String,
    mutations: Vec<Mutation>,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let body = serde_json::to_string(&mutations)?;
    let res = client
        .post(&format!("{}/mutations", server))
        .body(body)
        .header("Content-Type", "application/json")
        .header("Authorization", token)
        .send()
        .await?;

    println!("Sent mutation: {}", res.status());
    Ok(())
}

#[cfg(test)]
mod test {
    use super::create_mutations;
    use std::collections::HashMap;
    use std::fs;

    #[test]
    fn test_mutators() {
        let file = fs::read_to_string("mocks/mutators.json").expect("Unable to read file");
        let data: HashMap<String, Vec<String>> = serde_json::from_str(&file).unwrap();

        for (to_be_mutated, expected_results) in data {
            let mut mutations: Vec<(usize, String)> = vec![];
            let lines: Vec<&str> = vec![&to_be_mutated];
            let _success = true;
            create_mutations(&lines, &mut mutations);
            assert_eq!(mutations.len(), expected_results.len());

            for expected_result in expected_results {
                let mut find_result = false;
                for mutation in &mutations {
                    if mutation.1 == expected_result {
                        find_result = true;
                    }
                }
                assert_eq!(find_result, true);
            }
        }
    }
}
