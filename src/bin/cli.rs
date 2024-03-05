use clap::{Parser, Subcommand};
use hyparview::client::Client;
use tokio::net::{UnixListener, UnixStream};

#[derive(Parser, Debug)]
struct Cli {
    #[clap(subcommand)]
    command: Command,

    #[clap(name = "hostname", long, default_value = "127.0.0.1")]
    host: String,

    #[clap(long, default_value_t = 8080)]
    port: u16,
}

#[derive(Subcommand, Debug)]
enum Command {
    Join,
}

/// Entry point for CLI tool.
///
/// The `[tokio::main]` annotation signals that the Tokio runtime should be
/// started when the function is called. The body of the function is executed
/// within the newly spawned runtime.
///
/// `flavor = "current_thread"` is used here to avoid spawning background
/// threads. The CLI tool use case benefits more by being lighter instead of
/// multi-threaded.
#[tokio::main(flavor = "current_thread")]
async fn main() -> hyparview::Result<()> {
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
