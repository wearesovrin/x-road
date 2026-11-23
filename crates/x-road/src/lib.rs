//! X-Road Rust Workspace Facade & Umbrella Crate
//!
//! Re-exports all core X-Road libraries for ergonomic single-crate imports.

pub use acme;
pub use asic;
pub use central_server;
pub use common;
pub use confclient;
pub use crypto;
pub use globalconf;
pub use identifiers;
pub use identity;
pub use message;
pub use messagelog;
pub use monitor;
pub use proto;
pub use proxy;
pub use security_server;
pub use serverconf;
pub use signer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facade_reexports() {
        let client = identifiers::ClientId::member("DEV", "COM", "12345678").unwrap();
        assert_eq!(client.xroad_instance, "DEV");

        let hash = crypto::sha256_digest(b"test");
        assert_eq!(hash.len(), 32);

        let budget = common::CallBudget::new(std::time::Duration::from_secs(5));
        assert!(!budget.is_expired());
    }
}
