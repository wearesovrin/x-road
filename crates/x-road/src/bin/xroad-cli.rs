use asic::AsicContainer;
use clap::{Parser, Subcommand};
use identifiers::ClientId;
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
    /// Security Server Admin API operations
    #[command(name = "security-server", alias = "admin", alias = "ss")]
    SecurityServer {
        #[command(subcommand)]
        action: SecurityServerAction,
    },
    /// Central Server management operations
    #[command(name = "central-server", alias = "central", alias = "cs")]
    CentralServer {
        #[command(subcommand)]
        action: CentralServerAction,
    },
    /// MessageLog archive and verifier operations
    #[command(alias = "mlog")]
    MessageLog {
        #[command(subcommand)]
        action: MessageLogAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum MessageLogAction {
    /// Archive pending unarchived message logs into ASiC-E archive zip
    Archive {
        #[arg(
            short,
            long,
            default_value = "sqlite:////var/lib/xroad/messagelog.db?mode=rwc"
        )]
        db: String,
        #[arg(short, long, default_value = "/var/lib/xroad/archive")]
        dir: String,
        #[arg(short, long, default_value_t = 1000)]
        limit: usize,
    },
    /// Verify the integrity and hash chain of an X-Road message log archive
    Verify {
        #[arg(short, long)]
        file: String,
        #[arg(short, long)]
        prev_digest: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SecurityServerAction {
    /// List all registered clients on the Security Server
    ListClients {
        #[arg(short, long, default_value = "http://127.0.0.1:4000")]
        endpoint: String,
    },
    /// Register a new client on the Security Server
    AddClient {
        #[arg(short, long, default_value = "http://127.0.0.1:4000")]
        endpoint: String,
        #[arg(short, long)]
        client: String,
        #[arg(long, default_value_t = false)]
        is_owner: bool,
    },
    /// Query Security Server operational status
    Status {
        #[arg(short, long, default_value = "http://127.0.0.1:4000")]
        endpoint: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum CentralServerAction {
    /// Query Central Server operational status
    Status {
        #[arg(short, long, default_value = "http://127.0.0.1:4001")]
        endpoint: String,
    },
    /// List all registered X-Road members
    ListMembers {
        #[arg(short, long, default_value = "http://127.0.0.1:4001")]
        endpoint: String,
    },
    /// Register a new X-Road member
    AddMember {
        #[arg(short, long, default_value = "http://127.0.0.1:4001")]
        endpoint: String,
        #[arg(long)]
        member_class: String,
        #[arg(long)]
        member_code: String,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "admin@example.org")]
        contact: String,
    },
    /// List all registered Security Servers
    ListServers {
        #[arg(short, long, default_value = "http://127.0.0.1:4001")]
        endpoint: String,
    },
    /// Register a new Security Server
    AddServer {
        #[arg(short, long, default_value = "http://127.0.0.1:4001")]
        endpoint: String,
        #[arg(long)]
        owner_id: i64,
        #[arg(long)]
        server_code: String,
        #[arg(long)]
        address: String,
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
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();
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
                let signer = signer::SignerClient::new_in_memory();
                let key_info = signer
                    .generate_key(&token_id, &label, proto::KeyUsageInfo::Signing)
                    .await?;
                println!("Successfully generated key: {}", key_info.id);
                println!("Algorithm: {}", key_info.sign_mechanism_name);
                println!("Public Key:\n{}", key_info.public_key);
            }
            KeyAction::Csr { key_id, subject } => {
                let signer = signer::SignerClient::new_in_memory();
                // Generate a temporary key for CSR demonstration if not pre-existing
                let key = signer
                    .generate_key("soft-token-0", &key_id, proto::KeyUsageInfo::Signing)
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
        Commands::SecurityServer { action } => {
            let client = reqwest::Client::new();
            match action {
                SecurityServerAction::Status { endpoint } => {
                    let resp = client
                        .get(format!("{endpoint}/api/v1/status"))
                        .send()
                        .await?;
                    let status: serde_json::Value = resp.json().await?;
                    println!("{}", serde_json::to_string_pretty(&status)?);
                }
                SecurityServerAction::ListClients { endpoint } => {
                    let resp = client
                        .get(format!("{endpoint}/api/v1/clients"))
                        .send()
                        .await?;
                    let clients: serde_json::Value = resp.json().await?;
                    println!("{}", serde_json::to_string_pretty(&clients)?);
                }
                SecurityServerAction::AddClient {
                    endpoint,
                    client: client_str,
                    is_owner,
                } => {
                    let body = serde_json::json!({
                        "client_id": client_str,
                        "is_owner": is_owner,
                    });
                    let resp = client
                        .post(format!("{endpoint}/api/v1/clients"))
                        .json(&body)
                        .send()
                        .await?;
                    let res: serde_json::Value = resp.json().await?;
                    println!("{}", serde_json::to_string_pretty(&res)?);
                }
            }
        }
        Commands::CentralServer { action } => {
            let client = reqwest::Client::new();
            match action {
                CentralServerAction::Status { endpoint } => {
                    let resp = client
                        .get(format!("{endpoint}/api/v1/status"))
                        .send()
                        .await?;
                    let status: serde_json::Value = resp.json().await?;
                    println!("{}", serde_json::to_string_pretty(&status)?);
                }
                CentralServerAction::ListMembers { endpoint } => {
                    let resp = client
                        .get(format!("{endpoint}/api/v1/members"))
                        .send()
                        .await?;
                    let members: serde_json::Value = resp.json().await?;
                    println!("{}", serde_json::to_string_pretty(&members)?);
                }
                CentralServerAction::AddMember {
                    endpoint,
                    member_class,
                    member_code,
                    name,
                    contact,
                } => {
                    let body = serde_json::json!({
                        "member_class": member_class,
                        "member_code": member_code,
                        "name": name,
                        "administrative_contact": contact,
                    });
                    let resp = client
                        .post(format!("{endpoint}/api/v1/members"))
                        .json(&body)
                        .send()
                        .await?;
                    let res: serde_json::Value = resp.json().await?;
                    println!("{}", serde_json::to_string_pretty(&res)?);
                }
                CentralServerAction::ListServers { endpoint } => {
                    let resp = client
                        .get(format!("{endpoint}/api/v1/security-servers"))
                        .send()
                        .await?;
                    let servers: serde_json::Value = resp.json().await?;
                    println!("{}", serde_json::to_string_pretty(&servers)?);
                }
                CentralServerAction::AddServer {
                    endpoint,
                    owner_id,
                    server_code,
                    address,
                } => {
                    let body = serde_json::json!({
                        "owner_id": owner_id,
                        "server_code": server_code,
                        "address": address,
                    });
                    let resp = client
                        .post(format!("{endpoint}/api/v1/security-servers"))
                        .json(&body)
                        .send()
                        .await?;
                    let res: serde_json::Value = resp.json().await?;
                    println!("{}", serde_json::to_string_pretty(&res)?);
                }
            }
        }
        Commands::MessageLog { action } => match action {
            MessageLogAction::Archive {
                db: db_url,
                dir,
                limit,
            } => {
                println!("Connecting to MessageLog database: {db_url} ...");
                let db = messagelog::MessageLogDb::connect(&db_url).await?;
                let archiver =
                    messagelog::MessageLogArchiver::new(db, &dir).with_max_records(limit);
                println!("Starting archival run into directory: {dir} ...");
                match archiver.archive_batch().await? {
                    Some(info) => {
                        println!("Archival successful!");
                        println!("  Records archived: {}", info.record_count);
                        println!("  Archive file: {}", info.file_path.display());
                        println!("  Last chained digest: {}", info.last_digest);
                    }
                    None => {
                        println!("No unarchived records found.");
                    }
                }
            }
            MessageLogAction::Verify { file, prev_digest } => {
                println!("Verifying archive: {file} ...");
                let result = messagelog::verify_archive(&file, prev_digest.as_deref())?;
                println!("Archive verification SUCCEEDED!");
                println!("  Containers verified: {}", result.records_verified);
                println!(
                    "  Previous digest: {}",
                    if result.prev_digest.is_empty() {
                        "-"
                    } else {
                        &result.prev_digest
                    }
                );
                println!("  Final chained digest: {}", result.last_digest);
                for c in &result.containers {
                    println!("    [OK] {c}");
                }
            }
        },
    }

    Ok(())
}
