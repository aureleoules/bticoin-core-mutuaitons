use std::str::FromStr;

use crate::mutators::{
    execution_flow::ExecutionFlowMutator,
    operator::{BoolAritmeticMutator, BoolOperatorMutator, IncDecMutator, OperatorMutator},
    std_algorithm::StdAlgorithmMutator,
    Mutator, MutatorContext,
};
use common::{Mutation, MutationStatus};
use lazy_static::lazy_static;
use unidiff;

lazy_static! {
    static ref MUTATORS: Vec<Box<dyn Mutator + Sync>> = vec![
        Box::new(OperatorMutator::default()),
        Box::new(BoolOperatorMutator::default()),
        Box::new(ExecutionFlowMutator::default()),
        Box::new(StdAlgorithmMutator::default()),
        Box::new(BoolAritmeticMutator::default()),
        Box::new(IncDecMutator::default()),
    ];
}

pub fn generate_mutations(lines: &[&str]) -> Vec<(usize, String)> {
    let mut mutations = vec![];

    for (i, line) in lines.iter().enumerate() {
        mutations.extend(mutate_line(i, line));
    }

    mutations
}

pub fn mutate_line(number: usize, line: &str) -> Vec<(usize, String)> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//")
        || trimmed.starts_with('*')
        || trimmed.starts_with("assert")
        || trimmed.starts_with("/*")
        || trimmed.starts_with("LogPrint")
    {
        return vec![];
    }

    let mut mutations = vec![];

    for m in MUTATORS.iter() {
        let ctx = MutatorContext {
            file: "".to_string(),
            line: number,
            line_content: line.to_string(),
        };

        let muts = m.mutate(&ctx);
        mutations.extend(muts.into_iter().map(|m| (number, m)));
    }

    mutations
}

pub fn generate_mutations_from_files(files: &Vec<String>) -> Vec<Mutation> {
    let mut mutations = vec![];
    for file in files {
        println!("File: {}", file);

        let content = std::fs::read_to_string(&file).unwrap();
        let lines = content.split('\n').collect::<Vec<&str>>();

        let muts = generate_mutations(&lines);

        println!("{} mutations found", muts.len());
        println!("Generating patches...");

        for (line, mutation) in muts {
            let patch = create_patch(&content, line, &mutation);
            let m = create_mutation(file, &patch, line, Some("master".to_string()), None);
            mutations.push(m);
        }
    }

    mutations
}

fn create_patch(original_content: &str, line: usize, mutation: &str) -> String {
    let lines = original_content.split('\n').collect::<Vec<&str>>();

    let mut new_lines = vec![];
    for (i, l) in lines.iter().enumerate() {
        if i == line {
            new_lines.push(mutation);
        } else {
            new_lines.push(l);
        }
    }

    let new_content = new_lines.join("\n");
    let patch = diffy::create_patch(original_content, &new_content);

    patch.to_string()
}

fn create_mutation(
    file: &str,
    patch: &str,
    line: usize,
    branch: Option<String>,
    pr_number: Option<i64>,
) -> Mutation {
    let md5 = md5::compute(patch).to_vec();
    let md5 = hex::encode(md5);

    Mutation {
        id: 0,
        file: file.to_string(),
        patch_md5: md5,
        line: line as i64,
        patch: patch.to_string(),
        branch,
        pr_number,
        status: MutationStatus::Pending.to_string(),
        start_time: None,
        end_time: None,
        stderr: None,
        stdout: None,
    }
}

pub fn generate_mutations_from_pr(pr_number: i64) -> Vec<Mutation> {
    let cmd = std::process::Command::new("gh")
        .arg("pr")
        .arg("checkout")
        .arg(pr_number.to_string())
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8_lossy(&cmd.stdout);
    let output = output.trim();
    println!("Output: {}", output);

    std::process::Command::new("git")
        .arg("pull")
        .arg("origin")
        .arg("master")
        .arg("--rebase")
        .output()
        .expect("failed to execute process");

    let cmd = std::process::Command::new("git")
        .arg("diff")
        .arg("master")
        .output()
        .expect("failed to execute process");

    let output = String::from_utf8_lossy(&cmd.stdout);
    let output = output.trim();

    let patchset = unidiff::PatchSet::from_str(output).unwrap();

    let mut mutations = vec![];
    for patch in patchset {
        let file_path = patch.target_file.clone();
        let file_path = file_path[2..].to_string();
        if file_path.ends_with(".py") {
            continue;
        }
        if file_path.starts_with("test/")
            || file_path.starts_with("src/test")
            || file_path.starts_with("doc")
        {
            continue;
        }

        // remove a/ and b/ from the path
        println!("File path: {}", file_path);
        let file_content = std::fs::read_to_string(&file_path).unwrap();
        for hunk in patch {
            for line in hunk.lines() {
                if line.is_added() {
                    let muts = mutate_line(line.target_line_no.unwrap(), &line.value);
                    println!("Mutating line {}", line.value);
                    for (line_no, mutation) in muts {
                        println!("Mutation: {}", mutation);
                        let patch = create_patch(&file_content, line_no-1, &mutation);
                        let m = create_mutation(&file_path, &patch, line_no, None, Some(pr_number));

                        mutations.push(m);
                    }
                }
            }
        }
    }

    mutations
}

// #[cfg(test)]
// mod test {
//     use super::generate_mutations;
//     use std::collections::HashMap;
//     use std::fs;

//     #[test]
//     fn test_mutators() {
//         let file = fs::read_to_string("mocks/mutators.json").expect("Unable to read file");
//         let data: HashMap<String, Vec<String>> = serde_json::from_str(&file).unwrap();

//         for (to_be_mutated, expected_results) in data {
//             let lines: Vec<&str> = vec![&to_be_mutated];
//             let _success = true;
//             let mutations = generate_mutations(&lines);
//             println!("Mutations: {:?}", mutations);
//             println!("Expected: {:?}", expected_results);
//             assert_eq!(mutations.len(), expected_results.len());

//             for expected_result in expected_results {
//                 let mut find_result = false;
//                 for mutation in &mutations {
//                     if mutation.1 == expected_result {
//                         find_result = true;
//                     }
//                 }
//                 assert_eq!(find_result, true);
//             }
//         }
//     }
// }
