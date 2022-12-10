use clap::Parser;
mod run;

#[derive(Parser, Default)]
#[command(
    about = "Bitcoin Core Mutations",
    long_about = "Bticoin Core Mutuaitons is a tool for mutating Bitcoin Core's source code."
)]
struct Args {
    #[clap(short, long, help = "Server to get work from")]
    server: String,
    #[clap(
        short,
        long,
        help = "Path to Bitcoin Core",
        default_value = "/tmp/bitcoin"
    )]
    path: String,
    #[clap(short, long, help = "Build command", default_value = "make -j$(nproc)")]
    build_cmd: String,
    #[clap(
        short,
        long,
        help = "Test command",
        default_value = "make check -j$(nproc) && python3 -u test/functional/test_runner.py -j$(expr $(nproc) + 4) -F"
    )]
    test_cmd: String,
    #[clap(long, help = "Token to use for authentication")]
    token: String,
    #[clap(long, help = "Timeout (seconds)", default_value = "1800")]
    timeout: u64,
}

#[actix_web::main]
async fn main() {
    let args = Args::parse();

    ctrlc::set_handler(move || {
        println!("Bye!");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let r = run::execute_mutations(
        &args.server,
        &args.path,
        &args.build_cmd,
        &args.test_cmd,
        args.timeout,
        &args.token,
    )
    .await;

    if r.is_err() {
        panic!("Worker failed with error: {}", r.unwrap_err());
    }
}
