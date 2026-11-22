/// Configuration settings for the Security Server daemon and gateways.
#[derive(Debug, Clone)]
pub struct SecurityServerConfig {
    pub admin_bind: String,
    pub client_proxy_bind: String,
    pub server_proxy_bind: String,
    pub mtls_bind: String,
    pub enable_mtls: bool,
    pub database_url: Option<String>,
    pub messagelog_database_url: Option<String>,
    pub default_server_proxy_url: Option<String>,
    pub ui_static_path: Option<String>,
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
            mtls_bind: std::env::var("MTLS_BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:5500".to_string()),
            enable_mtls: std::env::var("XROAD_ENABLE_MTLS")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false),
            database_url: std::env::var("DATABASE_URL").ok(),
            messagelog_database_url: std::env::var("MESSAGELOG_DATABASE_URL").ok(),
            default_server_proxy_url: std::env::var("DEFAULT_SERVER_PROXY_URL").ok(),
            ui_static_path: std::env::var("XROAD_UI_PATH").ok(),
        }
    }
}
