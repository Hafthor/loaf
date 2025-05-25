use clap::Parser;
use loaf_lang::cli::{Cli, CliHandler};
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let handler = CliHandler::new();

    if let Err(e) = handler.handle(cli).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
