use strum::{AsRefStr, Display};
use thiserror::Error;

/// Result type for X-Road operations.
pub type XRoadResult<T> = Result<T, XRoadError>;

/// Standard X-Road fault codes.
#[derive(Debug, Clone, PartialEq, Eq, Display, AsRefStr)]
pub enum FaultCode {
    #[strum(to_string = "{0}")]
    Client(String),
    #[strum(to_string = "{0}")]
    Server(String),
}

impl FaultCode {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// Comprehensive X-Road error hierarchy.
#[derive(Error, Debug)]
pub enum XRoadError {
    #[error("Identifier parse error: {0}")]
    IdentifierParse(String),

    #[error("Identifier validation error: {0}")]
    IdentifierValidation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Cryptographic error: {0}")]
    Crypto(String),

    #[error("Signature verification error: {0}")]
    SignatureVerification(String),

    #[error("Certificate error: {0}")]
    Certificate(String),

    #[error("OCSP verification error: {0}")]
    Ocsp(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Message protocol error: {0}")]
    MessageProtocol(String),

    #[error("MIME multipart error: {0}")]
    Multipart(String),

    #[error("ASiC container error: {0}")]
    Asic(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Time budget expired")]
    Timeout,

    #[error("Internal fault ({code}): {message}")]
    InternalFault { code: String, message: String },
}

impl XRoadError {
    /// Maps the error to a standard X-Road SOAP / REST fault code.
    pub fn fault_code(&self) -> FaultCode {
        match self {
            XRoadError::IdentifierParse(_) => {
                FaultCode::Client("Client.InvalidIdentifier".to_string())
            }
            XRoadError::AccessDenied(_) => FaultCode::Client("Client.AccessDenied".to_string()),
            XRoadError::ServiceNotFound(_) => {
                FaultCode::Client("Client.ServiceNotFound".to_string())
            }
            XRoadError::MessageProtocol(_) => {
                FaultCode::Client("Client.InvalidMessage".to_string())
            }
            XRoadError::SignatureVerification(_) => {
                FaultCode::Server("Server.ServerProxy.InvalidSignature".to_string())
            }
            XRoadError::Certificate(_) => {
                FaultCode::Server("Server.ServerProxy.InvalidCertificate".to_string())
            }
            XRoadError::Ocsp(_) => FaultCode::Server("Server.ServerProxy.OcspError".to_string()),
            _ => FaultCode::Server("Server.ServerProxy.InternalError".to_string()),
        }
    }
}
