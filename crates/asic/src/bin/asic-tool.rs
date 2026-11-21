use asic::AsicContainer;
use clap::{Parser, Subcommand};
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "asic-tool")]
#[command(about = "ASiC-e Signature Container Packaging and Inspection Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Pack {
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
        Commands::Inspect { file } => {
            let data = fs::read(file)?;
            let container = AsicContainer::unpack(&data)?;
            println!(
                "ASiC-e message size: {} bytes",
                container.message_data.len()
            );
            println!(
                "ASiC-e signature size: {} bytes",
                container.signature_data.len()
            );
        }
    }

    Ok(())
}
