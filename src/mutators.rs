use fancy_regex::Regex;
use lazy_static::lazy_static;
use time::OffsetDateTime;

use crate::{Mutation, MutationStatus};

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

pub fn create_mutations(files: &Vec<String>) -> Vec<Mutation> {
    let mut mutations = Vec::new();
    for file in files {
        println!("File: {}", file);

        let content = std::fs::read_to_string(&file).unwrap();
        let lines = content.split('\n').collect::<Vec<&str>>();
        let mut muts: Vec<(usize, String)> = vec![]; // (line, mutation)

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//")
                || trimmed.starts_with('*')
                || trimmed.starts_with("assert")
            {
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
                id: md5,
                file: file.clone(),
                line,
                patch: patch.to_string(),
                branch: "master".to_string(),
                pr_number: None,
                status: MutationStatus::Pending,
                start_time: None,
                end_time: None,
            };

            mutations.push(m);
        }
    }

    mutations
}
pub async fn send_mutations(
    server: String,
    mutations: Vec<Mutation>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let body = serde_json::to_string(&mutations)?;
    let res = client
        .post(&format!("{}/mutations", server))
        .body(body)
        .header("Content-Type", "application/json")
        .send()
        .await?;

    println!("Sent mutation: {}", res.status());

    Ok(())
}
