pub mod client_proxy;
pub mod mtls;
pub mod server_proxy;

pub use client_proxy::{ClientProxyState, client_proxy_app, run_client_proxy};
pub use mtls::{create_server_mtls_config, generate_self_signed_cert, run_server_proxy_mtls};
pub use server_proxy::{ServerProxyState, run_server_proxy, server_proxy_app};

// Backward-compatible alias
pub use client_proxy::ClientProxyState as ProxyState;
