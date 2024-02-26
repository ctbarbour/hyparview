use hyparview::client::Client;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    #[clap(name = "hostname", long, default_value = "127.0.0.1")]
    host: String,

    #[clap(long, default_value_t = 8080)]
    port: u16
}

#[derive(Subcommand, Debug)]
enum Command {
    Join
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let addr = format!("{}:{}", cli.host, cli.port);

    let mut client = Client::connect(&addr).await?;

    match cli.command {
        Command::Join => {
            client.join().await?;
        }
    }

    Ok(())
}
