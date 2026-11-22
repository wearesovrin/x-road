pub mod client_proxy;
pub mod server_proxy;

pub use client_proxy::{ClientProxyState, client_proxy_app, run_client_proxy};
pub use server_proxy::{ServerProxyState, run_server_proxy, server_proxy_app};

// Backward-compatible alias
pub use client_proxy::ClientProxyState as ProxyState;
