use clap::{CommandFactory, Parser, Subcommand};
mod mutators;
mod run;
mod server;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Parser, Default)]
#[command(
    about = "Bitcoin Core Mutations",
    long_about = "Bticoin Core Mutuaitons is a tool for mutating Bitcoin Core's source code."
)]
struct Args {
    #[clap(subcommand)]
    action: Option<Action>,
}

#[derive(Subcommand)]
enum Action {
    #[clap(name = "mutate", about = "Mutate files")]
    Mutate {
        #[clap(short, long, help = "Files to mutate")]
        files: Vec<String>,
        #[clap(short, long, help = "Server to send mutations")]
        server: String,
    },
    #[clap(name = "server", about = "Start the server")]
    Server {
        #[clap(long, help = "Host", default_value = "0.0.0.0")]
        host: String,
        #[clap(long, help = "Port", default_value = "8080")]
        port: u16,
        #[clap(long, help = "DB path", default_value = "db")]
        db: String,
    },
    #[clap(name = "run", about = "Run mutations")]
    Run {
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
            default_value = "make check -j$(expr $(nproc) + 4) && test/functional/test_runner.py -j$(expr $(nproc) + 4) -F"
        )]
        test_cmd: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MutationStatus {
    Pending,
    Timeout,
    Running,
    Killed,
    NotKilled,
    Ignored,
    Error,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Mutation {
    id: String,
    file: String,
    line: usize,
    patch: String,
    branch: String,
    pr_number: Option<String>,
    status: MutationStatus,
    #[serde(default, with = "time::serde::timestamp::option")]
    start_time: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::timestamp::option")]
    end_time: Option<OffsetDateTime>,
}

#[actix_web::main]
async fn main() {
    let args = Args::parse();

    ctrlc::set_handler(move || {
        println!("Bye!");
        std::process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    match &args.action {
        Some(Action::Mutate { server, files }) => {
            if files.is_empty() {
                println!("No files to mutate, please specify some files with --files.");
                return;
            }

            let mutations = mutators::create_mutations(files);
            mutators::send_mutations(server.to_string(), mutations)
                .await
                .unwrap();
        }
        Some(Action::Server { host, port, db }) => {
            server::run(host.clone(), *port, db.clone()).await;
        }
        Some(Action::Run {
            server,
            path,
            build_cmd,
            test_cmd,
        }) => {
            run::execute_mutations(server, path, build_cmd, test_cmd).await;
        }
        None => {
            let mut cmd = Args::command();
            cmd.print_help().unwrap_or(());
        }
    }
}
