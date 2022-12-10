use clap::Parser;
use common::Mutation;
pub mod mutate;
pub mod mutators;

#[derive(Parser, Default)]
#[command(
    about = "Bitcoin Core Mutations",
    long_about = "Bticoin Core Mutuaitons is a tool for mutating Bitcoin Core's source code."
)]
struct Args {
    #[clap(short, long, help = "Files to mutate")]
    files: Vec<String>,
    #[clap(short, long, help = "Server to send mutations")]
    server: String,
    #[clap(long, help = "Token to use for authentication")]
    token: String,
    #[clap(long, help = "Debug mode")]
    debug: bool,
}

#[actix_web::main]
async fn main() {
    let args = Args::parse();

    let files = args.files;

    if files.is_empty() {
        println!("No files to mutate, please specify some files with --files.");
        return;
    }

    let mutations = mutate::generate_mutations_from_files(&files);

    if args.debug {
        for m in &mutations {
            println!("{}", m.patch);
            println!("---\n");
        }

        return;
    }

    let r = send_mutations(args.server, mutations, &args.token).await;

    if r.is_err() {
        panic!("Mutator failed with error: {}", r.unwrap_err());
    }
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
