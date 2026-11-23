use common::XRoadResult;
use sha2::{Digest, Sha256, Sha384, Sha512};
use std::future::Future;

/// Supported cryptographic digest algorithms in X-Road.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DigestAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl DigestAlgorithm {
    /// Returns the standard canonical algorithm name.
    pub fn digest_name(&self) -> &'static str {
        match self {
            Self::Sha256 => "SHA-256",
            Self::Sha384 => "SHA-384",
            Self::Sha512 => "SHA-512",
        }
    }

    /// Returns the W3C XML-DSig / XML-Enc identifier URI for this digest algorithm.
    pub fn uri(&self) -> &'static str {
        match self {
            Self::Sha256 => "http://www.w3.org/2001/04/xmlenc#sha256",
            Self::Sha384 => "http://www.w3.org/2001/04/xmldsig-more#sha384",
            Self::Sha512 => "http://www.w3.org/2001/04/xmlenc#sha512",
        }
    }

    /// Resolves an XML-DSig URI to a `DigestAlgorithm`.
    pub fn from_uri(uri: &str) -> Option<Self> {
        match uri.trim() {
            "http://www.w3.org/2001/04/xmlenc#sha256"
            | "http://www.w3.org/2000/09/xmldsig#sha256" => Some(Self::Sha256),
            "http://www.w3.org/2001/04/xmldsig-more#sha384" => Some(Self::Sha384),
            "http://www.w3.org/2001/04/xmlenc#sha512"
            | "http://www.w3.org/2000/09/xmldsig#sha512" => Some(Self::Sha512),
            _ => None,
        }
    }

    /// Resolves a human-readable or HTTP header algorithm name to a `DigestAlgorithm`.
    pub fn from_name(name: &str) -> Option<Self> {
        let cleaned = name.trim().to_ascii_uppercase().replace('-', "");
        match cleaned.as_str() {
            "SHA256" => Some(Self::Sha256),
            "SHA384" => Some(Self::Sha384),
            "SHA512" => Some(Self::Sha512),
            _ => None,
        }
    }

    /// Computes the cryptographic digest of the input data using this algorithm.
    pub fn digest(&self, data: &[u8]) -> Vec<u8> {
        match self {
            Self::Sha256 => sha256_digest(data).to_vec(),
            Self::Sha384 => sha384_digest(data).to_vec(),
            Self::Sha512 => sha512_digest(data).to_vec(),
        }
    }
}

/// Computes the cryptographic digest of data using SHA-256.
pub fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Computes the cryptographic digest of data using SHA-384.
pub fn sha384_digest(data: &[u8]) -> [u8; 48] {
    let mut hasher = Sha384::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Computes the cryptographic digest of data using SHA-512.
pub fn sha512_digest(data: &[u8]) -> [u8; 64] {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Encodes a binary byte slice into a lowercase hexadecimal string.
pub fn to_hex(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for b in data {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

/// Decodes a hexadecimal string into binary bytes.
pub fn from_hex(hex: &str) -> Result<Vec<u8>, String> {
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return Err("Hex string length must be even".to_string());
    }

    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex byte at index {i}: {e}"))
        })
        .collect()
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

        let d384 = sha384_digest(data);
        assert_eq!(d384.len(), 48);

        let d512 = sha512_digest(data);
        assert_eq!(d512.len(), 64);
    }

    #[test]
    fn test_digest_algorithm_properties() {
        assert_eq!(DigestAlgorithm::Sha256.digest_name(), "SHA-256");
        assert_eq!(
            DigestAlgorithm::Sha256.uri(),
            "http://www.w3.org/2001/04/xmlenc#sha256"
        );
        assert_eq!(
            DigestAlgorithm::from_uri("http://www.w3.org/2001/04/xmlenc#sha256"),
            Some(DigestAlgorithm::Sha256)
        );
        assert_eq!(
            DigestAlgorithm::from_uri("http://www.w3.org/2001/04/xmldsig-more#sha384"),
            Some(DigestAlgorithm::Sha384)
        );
        assert_eq!(
            DigestAlgorithm::from_uri("http://www.w3.org/2001/04/xmlenc#sha512"),
            Some(DigestAlgorithm::Sha512)
        );
        assert_eq!(DigestAlgorithm::from_uri("unknown-uri"), None);

        assert_eq!(
            DigestAlgorithm::from_name("SHA-256"),
            Some(DigestAlgorithm::Sha256)
        );
        assert_eq!(
            DigestAlgorithm::from_name("sha384"),
            Some(DigestAlgorithm::Sha384)
        );
        assert_eq!(
            DigestAlgorithm::from_name("SHA512"),
            Some(DigestAlgorithm::Sha512)
        );

        let digest = DigestAlgorithm::Sha256.digest(b"test");
        assert_eq!(digest.len(), 32);
    }

    #[test]
    fn test_hex_conversions() {
        let bytes = vec![0xde, 0xad, 0xbe, 0xef];
        let hex = to_hex(&bytes);
        assert_eq!(hex, "deadbeef");

        let decoded = from_hex(&hex).unwrap();
        assert_eq!(decoded, bytes);

        assert!(from_hex("abc").is_err()); // odd length
        assert!(from_hex("invalid!").is_err()); // non-hex char
    }
}
