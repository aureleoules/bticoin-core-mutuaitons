use std::io::{BufRead, BufReader};

use wait_timeout::ChildExt;

use common::{Mutation, MutationResult, MutationStatus};

pub async fn execute_mutations(
    server: &str,
    path: &str,
    build_cmd: &str,
    test_cmd: &str,
    timeout: u64,
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
        if res.status() == 204 {
            println!("No work available");
            std::thread::sleep(std::time::Duration::from_secs(120));
            continue;
        }

        let mutation = serde_json::from_str::<Mutation>(&res.text().await?)?;
        println!("Got work: {}", mutation.id);

        // Execute shell commands
        let mut cmd = std::process::Command::new("bash");
        cmd.current_dir(path);
        cmd.arg("-c");
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let branch = if mutation.branch.is_some() {
            mutation.branch.unwrap()
        } else {
            todo!("Handle this case")
        };

        let mut cmd_str = format!(
            "git stash && git checkout {} && git pull origin {}",
            branch, branch
        );

        // Store patch
        let patch_path = format!("/tmp/{}.patch", mutation.id);
        std::fs::write(&patch_path, mutation.patch)?;
        cmd_str = format!("{} && patch {} {}", cmd_str, mutation.file, patch_path);
        cmd_str = format!("{} && {}", cmd_str, build_cmd);
        cmd_str = format!("{} && {}", cmd_str, test_cmd);
        cmd.arg(cmd_str.clone());

        let mut child = cmd.spawn().unwrap();

        // Stream stdout and stderr
        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let stdout = BufReader::new(stdout);
        let stderr = BufReader::new(stderr);

        // separate thread to read stdout and stderr
        let stdout_handle = std::thread::spawn(move || {
            let mut stdout_str = String::new();
            for line in stdout.lines() {
                let line = line.unwrap();
                stdout_str = format!("{}\n{}", stdout_str, line);
                println!("stdout: {}", line);
            }

            stdout_str
        });

        let stderr_handle = std::thread::spawn(move || {
            let mut stderr_str = String::new();
            for line in stderr.lines() {
                let line = line.unwrap();
                stderr_str = format!("{}\n{}", stderr_str, line);
                println!("stderr: {}", line);
            }

            stderr_str
        });

        let status = match child
            .wait_timeout(std::time::Duration::from_secs(timeout))
            .unwrap()
        {
            Some(status) => status.code(),
            None => {
                println!("Timeout reached, killing process");
                child.kill().unwrap();

                None
            }
        };

        let stdout_str = stdout_handle.join().unwrap();
        let stderr_str = stderr_handle.join().unwrap();

        let status = match status {
            Some(0) => MutationStatus::NotKilled,
            None => MutationStatus::Timeout,
            _ => MutationStatus::Killed,
        };

        println!("Mutation {} status: {:?}", mutation.id, status);

        let client = reqwest::Client::new();
        let res = client
            .post(&format!("{}/mutations/{}", server, mutation.id))
            .body(serde_json::to_string(&MutationResult {
                mutation_id: mutation.patch_md5,
                status,
                stdout: Some(stdout_str),
                stderr: Some(stderr_str),
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
