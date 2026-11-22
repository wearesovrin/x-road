/// Configuration settings for the Security Server daemon and gateways.
#[derive(Debug, Clone)]
pub struct SecurityServerConfig {
    pub admin_bind: String,
    pub client_proxy_bind: String,
    pub server_proxy_bind: String,
    pub database_url: Option<String>,
    pub messagelog_database_url: Option<String>,
    pub default_server_proxy_url: Option<String>,
}

impl Default for SecurityServerConfig {
    fn default() -> Self {
        Self {
            admin_bind: std::env::var("ADMIN_BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:4000".to_string()),
            client_proxy_bind: std::env::var("CLIENT_PROXY_BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8000".to_string()),
            server_proxy_bind: std::env::var("SERVER_PROXY_BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8443".to_string()),
            database_url: std::env::var("DATABASE_URL").ok(),
            messagelog_database_url: std::env::var("MESSAGELOG_DATABASE_URL").ok(),
            default_server_proxy_url: std::env::var("DEFAULT_SERVER_PROXY_URL").ok(),
        }
    }
}
