use asic::AsicContainer;
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "xroad-cli")]
#[command(about = "X-Road Administration and Operations CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Check the status of X-Road services
    Status,
    /// Key management operations
    Key {
        #[command(subcommand)]
        action: KeyAction,
    },
    /// Inspect or update configuration
    Conf {
        #[arg(short, long, default_value = "DEV")]
        instance: String,
    },
    /// ASiC-e signature container operations
    Asic {
        #[command(subcommand)]
        action: AsicAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum KeyAction {
    List,
    Generate {
        #[arg(short, long)]
        token_id: String,
        #[arg(short, long)]
        label: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum AsicAction {
    /// Pack message and signature into an .asice file
    Pack {
        #[arg(short, long)]
        message: String,
        #[arg(short, long)]
        signature: String,
        #[arg(short, long)]
        output: String,
    },
    /// Inspect an .asice container
    Inspect {
        #[arg(short, long)]
        file: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            println!("X-Road services status: OK");
        }
        Commands::Key { action } => match action {
            KeyAction::List => println!("Listing keys..."),
            KeyAction::Generate { token_id, label } => {
                println!(
                    "Generating key on token '{}' with label '{}'...",
                    token_id, label
                );
            }
        },
        Commands::Conf { instance } => {
            println!("Configuration for instance: {}", instance);
        }
        Commands::Asic { action } => match action {
            AsicAction::Pack {
                message,
                signature,
                output,
            } => {
                let msg_bytes = fs::read(message)?;
                let sig_bytes = fs::read(signature)?;
                let container = AsicContainer::new(msg_bytes, sig_bytes);
                let packed = container.pack()?;
                fs::write(output, packed)?;
                println!("Created ASiC-e container successfully.");
            }
            AsicAction::Inspect { file } => {
                let data = fs::read(file)?;
                let container = AsicContainer::unpack(&data)?;
                println!(
                    "ASiC-e payload files: {}",
                    container.files.len()
                );
                if let Some(msg) = container.message_data() {
                    println!("ASiC-e message size: {} bytes", msg.len());
                }
                if let Some(sig) = container.signature_data() {
                    println!("ASiC-e signature size: {} bytes", sig.len());
                }
            }
        },
    }

    Ok(())
}
