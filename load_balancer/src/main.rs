use std::{path::PathBuf, sync::Arc};

use clap::{arg, command, Parser};
use lb::{
    config::{Config},
    policies::RoundRobinPolicy,
    Error, LoadBalancer,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long)]
    port: Option<u16>,

    #[arg(short, long)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();

    let port = args.port.unwrap_or(8080u16);
    let config = Config::new(&args.config)?;

    let policy = Arc::new(RoundRobinPolicy::new(&config));

    println!("Backends configured:");
    for backend in &config.backends {
        println!(
            "  - {}, healthcheck on {}",
            backend.url, backend.healthcheck_path
        );
    }
    println!();

    println!("Listening on 127.0.0.1:{}", port);

    LoadBalancer::new(port, config, policy).run().await;

    Ok(())
}
