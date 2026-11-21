use common::XRoadResult;
use sha2::{Digest, Sha256, Sha512};
use std::future::Future;

/// Supported digest algorithms in X-Road.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigestAlgorithm {
    Sha256,
    Sha512,
}

/// Computes the cryptographic digest of data using SHA-256.
pub fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Computes the cryptographic digest of data using SHA-512.
pub fn sha512_digest(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Cryptographic Signer trait for signing digests and managing keys.
pub trait Signer: Send + Sync {
    /// Returns the unique key ID.
    fn key_id(&self) -> &str;

    /// Signs a precomputed digest using the designated algorithm.
    fn sign(
        &self,
        algorithm: &str,
        digest: &[u8],
    ) -> impl Future<Output = XRoadResult<Vec<u8>>> + Send;

    /// Returns the certificate chain for this signing key.
    fn certificate_chain(&self) -> impl Future<Output = XRoadResult<Vec<Vec<u8>>>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digests() {
        let data = b"hello x-road";
        let d256 = sha256_digest(data);
        assert_eq!(d256.len(), 32);

        let d512 = sha512_digest(data);
        assert_eq!(d512.len(), 64);
    }
}
