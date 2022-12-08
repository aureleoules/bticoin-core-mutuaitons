use clap::Parser;
mod mutator;

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
}

#[actix_web::main]
async fn main() {
    let args = Args::parse();

    let files = args.files;

    if files.is_empty() {
        println!("No files to mutate, please specify some files with --files.");
        return;
    }

    let mutations = mutator::create_mutations_from_files(&files);

    let r = mutator::send_mutations(args.server, mutations, &args.token).await;

    if r.is_err() {
        panic!("Mutator failed with error: {}", r.unwrap_err());
    }
}
