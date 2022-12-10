use crate::mutators::{
    execution_flow::ExecutionFlowMutator,
    operator::{BoolAritmeticMutator, BoolOperatorMutator, IncDecMutator, OperatorMutator},
    std_algorithm::StdAlgorithmMutator,
    Mutator, MutatorContext,
};
use common::{Mutation, MutationStatus};
use lazy_static::lazy_static;

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

pub fn generate_mutations(content: String, lines: &[&str]) -> Vec<(usize, String)> {
    let mut mutations = vec![];

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//")
            || trimmed.starts_with('*')
            || trimmed.starts_with("assert")
            || trimmed.starts_with("/*")
            || trimmed.starts_with("LogPrint")
        {
            continue;
        }

        for m in MUTATORS.iter() {
            let ctx = MutatorContext {
                file: content.clone(),
                line: i,
                line_content: line.to_string(),
            };

            let muts = m.mutate(&ctx);
            mutations.extend(muts.into_iter().map(|m| (i, m)));
        }
    }

    mutations
}

pub fn generate_mutations_from_files(files: &Vec<String>) -> Vec<Mutation> {
    let mut mutations = vec![];
    for file in files {
        println!("File: {}", file);

        let content = std::fs::read_to_string(&file).unwrap();
        let lines = content.split('\n').collect::<Vec<&str>>();

        let muts = generate_mutations(content.clone(), &lines);

        println!("{} mutations found", muts.len());

        println!("Generating patches...");
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
                file: file.clone(),
                patch_md5: md5,
                line: line as i64,
                patch: patch.to_string(),
                branch: Some("master".to_string()),
                pr_number: None,
                status: MutationStatus::Pending.to_string(),
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
