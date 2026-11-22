use crate::config::SecurityServerConfig;
use crate::handlers::{AdminApiState, admin_api_app};
use common::XRoadResult;
use globalconf::GlobalConf;
use messagelog::MessageLogDb;
use poem::listener::TcpListener;
use poem::{Route, Server};
use proxy::{ClientProxyState, ServerProxyState, client_proxy_app, server_proxy_app};
use serverconf::DatabaseServerConf;
use signer::SignerServiceImpl;
use std::sync::Arc;

/// Unified X-Road Security Server daemon and gateway orchestrator.
#[derive(Clone)]
pub struct SecurityServerNode {
    pub config: SecurityServerConfig,
    pub server_conf: Arc<DatabaseServerConf>,
    pub message_log: Arc<MessageLogDb>,
    pub global_conf: Arc<GlobalConf>,
    pub signer: Arc<SignerServiceImpl>,
    pub signer_client: Arc<signer::SignerClient>,
}

impl SecurityServerNode {
    /// Creates a new `SecurityServerNode` with provided databases and configurations.
    pub fn new(
        config: SecurityServerConfig,
        server_conf: DatabaseServerConf,
        message_log: MessageLogDb,
        global_conf: GlobalConf,
    ) -> Self {
        let signer_service = Arc::new(SignerServiceImpl::new());
        let signer_client = Arc::new(signer::SignerClient::in_memory(signer_service.clone()));
        Self {
            config,
            server_conf: Arc::new(server_conf),
            message_log: Arc::new(message_log),
            global_conf: Arc::new(global_conf),
            signer: signer_service,
            signer_client,
        }
    }

    /// Initializes a `SecurityServerNode` with in-memory SQLite databases for testing and zero-config dev.
    pub async fn new_in_memory(config: SecurityServerConfig) -> XRoadResult<Self> {
        let server_conf = DatabaseServerConf::connect_in_memory().await?;
        let message_log = MessageLogDb::connect_in_memory().await?;
        let global_conf = GlobalConf::new("DEV");
        Ok(Self::new(config, server_conf, message_log, global_conf))
    }

    /// Connects to databases specified in config, or falls back to in-memory SQLite.
    pub async fn connect_from_config(config: SecurityServerConfig) -> XRoadResult<Self> {
        let server_conf = match config.database_url.as_deref() {
            Some(url) => DatabaseServerConf::connect(url).await?,
            None => DatabaseServerConf::connect_in_memory().await?,
        };

        let message_log = match config.messagelog_database_url.as_deref() {
            Some(url) => MessageLogDb::connect(url).await?,
            None => MessageLogDb::connect_in_memory().await?,
        };

        let global_conf = GlobalConf::new("DEV");
        Ok(Self::new(config, server_conf, message_log, global_conf))
    }

    /// Constructs the Poem Route for the Client Proxy (outgoing consumer gateway).
    pub fn client_proxy_app(&self) -> Route {
        let mut state = ClientProxyState::new(
            self.server_conf.clone(),
            self.message_log.clone(),
            self.global_conf.clone(),
        )
        .with_signer_client(self.signer_client.clone());

        if let Some(ref def) = self.config.default_server_proxy_url {
            state = state.with_default_server_proxy(def);
        }
        client_proxy_app(state)
    }

    /// Constructs the Poem Route for the Server Proxy (incoming provider gateway).
    pub fn server_proxy_app(&self) -> Route {
        let state = ServerProxyState::new(self.server_conf.clone(), self.message_log.clone())
            .with_signer_client(self.signer_client.clone());

        server_proxy_app(state)
    }

    /// Constructs the Poem Route for the Admin REST API & Swagger UI.
    pub fn admin_api_app(&self) -> Route {
        let state = AdminApiState::new(
            self.server_conf.clone(),
            self.message_log.clone(),
            self.global_conf.clone(),
            self.signer.clone(),
        );
        admin_api_app(state)
    }

    /// Runs all Security Server services concurrently (Admin REST API, Client Proxy, Server Proxy).
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!(
            admin = %self.config.admin_bind,
            client_proxy = %self.config.client_proxy_bind,
            server_proxy = %self.config.server_proxy_bind,
            "Starting unified X-Road Security Server node..."
        );

        let admin_listener = TcpListener::bind(&self.config.admin_bind);
        let client_listener = TcpListener::bind(&self.config.client_proxy_bind);
        let server_listener = TcpListener::bind(&self.config.server_proxy_bind);

        let admin_server = Server::new(admin_listener).run(self.admin_api_app());
        let client_server = Server::new(client_listener).run(self.client_proxy_app());
        let server_proxy_server = Server::new(server_listener).run(self.server_proxy_app());

        tokio::try_join!(
            async {
                admin_server
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            },
            async {
                client_server
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            },
            async {
                server_proxy_server
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }
        )?;

        Ok(())
    }
}
