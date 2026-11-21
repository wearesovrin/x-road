pub mod client_proxy;
pub mod server_proxy;

pub use client_proxy::{ClientProxyState, client_proxy_app, run_client_proxy};
pub use server_proxy::{ServerProxyState, server_proxy_app, run_server_proxy};

// Backward-compatible alias
pub use client_proxy::ClientProxyState as ProxyState;
