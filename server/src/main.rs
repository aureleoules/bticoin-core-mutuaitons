use clap::Parser;
mod server;

#[derive(Parser, Default)]
#[command(
    about = "Bitcoin Core Mutations",
    long_about = "Bticoin Core Mutuaitons is a tool for mutating Bitcoin Core's source code."
)]
struct Args {
    #[clap(long, help = "Host", default_value = "0.0.0.0")]
    host: String,
    #[clap(long, help = "Port", default_value = "8080")]
    port: u16,
    #[clap(long, help = "Redis database", default_value = "127.0.0.1")]
    redis: String,
    #[clap(
        long = "token",
        help = "Authorized tokens (owner:token)",
        required = true
    )]
    tokens: Vec<String>,
}

#[actix_web::main]
async fn main() {
    let args = Args::parse();
    if server::run(args.host, args.port, args.redis, args.tokens)
        .await
        .is_err()
    {
        panic!("Failed to start server");
    }
}
