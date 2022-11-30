use std::{
    io::{BufRead, BufReader},
    os::unix::process::ExitStatusExt,
    process::ExitStatus,
};

use time::OffsetDateTime;

use crate::{Mutation, MutationResult, MutationStatus};

pub async fn execute_mutations(
    server: &str,
    path: &str,
    build_cmd: &str,
    test_cmd: &str,
    token: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Running mutations...");

    loop {
        println!("Getting work...");
        let client = reqwest::Client::new();
        let res = client
            .post(&format!("{}/get_work", server))
            .header("Authorization", token)
            .send()
            .await?;
        println!("Got work: {:?}", res);
        if res.status() == 204 {
            println!("No work available");
            std::thread::sleep(std::time::Duration::from_secs(60));
            continue;
        }

        let mutation = serde_json::from_str::<Mutation>(&res.text().await?)?;

        // Execute shell commands
        let mut cmd = std::process::Command::new("bash");
        cmd.current_dir(path);
        cmd.arg("-c");
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut cmd_str = format!("git stash && git checkout {}", mutation.branch);

        // Store patch
        let patch_path = format!("/tmp/{}.patch", mutation.id);
        std::fs::write(&patch_path, mutation.patch)?;
        cmd_str = format!("{} && patch {} {}", cmd_str, mutation.file, patch_path);
        cmd_str = format!("{} && {}", cmd_str, build_cmd);
        cmd_str = format!("{} && {}", cmd_str, test_cmd);
        cmd.arg(cmd_str.clone());

        println!("Executing: {}", cmd_str);

        let start_time = OffsetDateTime::now_utc();
        // Timeout after 1h
        let timeout_t = std::time::Duration::from_secs(60 * 90); // 90 minutes

        // Get stderr and stdout

        let mut child = cmd.spawn()?;

        let mut timeout = false;
        let mut status_code = ExitStatus::from_raw(-1);
        loop {
            if start_time + timeout_t < OffsetDateTime::now_utc() {
                println!("Timeout");
                child.kill()?;
                timeout = true;
                break;
            }

            match child.try_wait()? {
                Some(status) => {
                    println!("Child exited with status: {}", status);
                    status_code = status;
                    break;
                }
                None => {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }
        }

        let stdout = BufReader::new(child.stdout.take().unwrap())
            .lines()
            .map(|l| l.unwrap())
            .collect::<Vec<String>>()
            .join("\n");

        let stderr = BufReader::new(child.stderr.take().unwrap())
            .lines()
            .map(|l| l.unwrap())
            .collect::<Vec<String>>()
            .join("\n");

        let status = if timeout {
            MutationStatus::Timeout
        } else if status_code.success() {
            MutationStatus::NotKilled
        } else {
            MutationStatus::Killed
        };

        println!("Mutation {} status: {:?}", mutation.id, status);

        let client = reqwest::Client::new();
        let res = client
            .post(&format!("{}/mutations/{}", server, mutation.id))
            .body(serde_json::to_string(&MutationResult {
                mutation_id: mutation.id,
                status,
                stdout,
                stderr,
            })?)
            .header("Content-Type", "application/json")
            .header("Authorization", token)
            .send()
            .await;

        if let Err(e) = res {
            println!("Error sending mutation status: {}", e);
        } else {
            println!("Mutation status sent: {}", res.unwrap().status());
        }
    }
}
