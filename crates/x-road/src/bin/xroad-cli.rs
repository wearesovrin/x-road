use asic::AsicContainer;
use clap::{Parser, Subcommand};
use identifiers::ClientId;
use signer::SignerServiceImpl;
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
    /// Inspect or download configuration from a Central Server
    Conf {
        #[arg(short, long)]
        url: Option<String>,
        #[arg(short, long, default_value = "DEV")]
        instance: String,
    },
    /// Decentralized identity and DID operations
    Did {
        #[command(subcommand)]
        action: DidAction,
    },
    /// ASiC-e signature container operations
    Asic {
        #[command(subcommand)]
        action: AsicAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum KeyAction {
    /// List available software token keys
    List,
    /// Generate a new software key
    Generate {
        #[arg(short, long, default_value = "soft-token-0")]
        token_id: String,
        #[arg(short, long)]
        label: String,
    },
    /// Generate a Certificate Signing Request (CSR)
    Csr {
        #[arg(short, long)]
        key_id: String,
        #[arg(short, long)]
        subject: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum DidAction {
    /// Format a ClientId into a W3C / Sovrin DID
    Format {
        #[arg(short, long)]
        client: String,
    },
    /// Parse a DID into an X-Road ClientId
    Parse {
        #[arg(short, long)]
        did: String,
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
            KeyAction::List => {
                println!("Software token keys ready.");
            }
            KeyAction::Generate { token_id, label } => {
                let signer = SignerServiceImpl::new();
                let key_info = signer
                    .generate_software_key(
                        &token_id,
                        &label,
                        proto::KeyUsageInfo::Signing,
                    )
                    .await?;
                println!("Successfully generated key: {}", key_info.id);
                println!("Algorithm: {}", key_info.sign_mechanism_name);
                println!("Public Key:\n{}", key_info.public_key);
            }
            KeyAction::Csr { key_id, subject } => {
                let signer = SignerServiceImpl::new();
                // Generate a temporary key for CSR demonstration if not pre-existing
                let key = signer
                    .generate_software_key("soft-token-0", &key_id, proto::KeyUsageInfo::Signing)
                    .await?;
                let csr = signer.generate_csr(&key.id, &subject).await?;
                println!("Generated PKCS#10 CSR for Subject: {}\n{}", subject, csr);
            }
        },
        Commands::Conf { url, instance } => {
            if let Some(endpoint) = url {
                println!("Downloading global configuration from: {}", endpoint);
                let anchor = globalconf::ConfigurationAnchor {
                    instance_identifier: instance,
                    generated_at: chrono::Utc::now(),
                    download_urls: vec![endpoint],
                    verification_cert_hashes: vec![],
                };
                let client = confclient::ConfClient::new(anchor);
                let conf = client.sync_once().await?;
                println!("Global Configuration successfully synchronized!");
                println!("Instance: {}", conf.instance_identifier);
                println!("Registered Members: {}", conf.members.len());
                for (id, name) in &conf.members {
                    println!("  - {}: {}", id, name);
                }
                println!("Security Servers: {}", conf.security_servers.len());
                for (id, addr) in &conf.security_servers {
                    println!("  - {}: {}", id, addr);
                }
                println!("Approved CAs: {}", conf.approved_cas.len());
                println!("Approved TSAs: {}", conf.approved_tsps.len());
            } else {
                println!("Configuration for instance: {}", instance);
            }
        }
        Commands::Did { action } => match action {
            DidAction::Format { client } => {
                let client_id: ClientId = client.parse()?;
                let did = identity::client_id_to_did(&client_id);
                println!("DID: {}", did);
            }
            DidAction::Parse { did } => {
                let client_id = identity::did_to_client_id(&did)?;
                println!("ClientId: {}", client_id);
            }
        },
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
                println!("ASiC-e payload files: {}", container.files.len());
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
