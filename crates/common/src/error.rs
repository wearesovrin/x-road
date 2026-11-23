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

    #[error("Message log error: {0}")]
    MessageLog(String),

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
            XRoadError::IdentifierParse(_) | XRoadError::IdentifierValidation(_) => {
                FaultCode::Client("Client.InvalidIdentifier".to_string())
            }
            XRoadError::AccessDenied(_) => FaultCode::Client("Client.AccessDenied".to_string()),
            XRoadError::ServiceNotFound(_) => {
                FaultCode::Client("Client.ServiceNotFound".to_string())
            }
            XRoadError::MessageProtocol(_) => {
                FaultCode::Client("Client.InvalidMessage".to_string())
            }
            XRoadError::Multipart(_) => FaultCode::Client("Client.InvalidMultipart".to_string()),
            XRoadError::Asic(_) => FaultCode::Client("Client.InvalidAsic".to_string()),
            XRoadError::SignatureVerification(_) => {
                FaultCode::Server("Server.ServerProxy.InvalidSignature".to_string())
            }
            XRoadError::Certificate(_) => {
                FaultCode::Server("Server.ServerProxy.InvalidCertificate".to_string())
            }
            XRoadError::Ocsp(_) => FaultCode::Server("Server.ServerProxy.OcspError".to_string()),
            XRoadError::Timeout => FaultCode::Server("Server.ServerProxy.Timeout".to_string()),
            XRoadError::InternalFault { code, .. } => FaultCode::Server(code.clone()),
            _ => FaultCode::Server("Server.ServerProxy.InternalError".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xroad_error_fault_codes() {
        assert_eq!(
            XRoadError::IdentifierParse("bad id".into()).fault_code(),
            FaultCode::Client("Client.InvalidIdentifier".into())
        );
        assert_eq!(
            XRoadError::IdentifierValidation("bad char".into()).fault_code(),
            FaultCode::Client("Client.InvalidIdentifier".into())
        );
        assert_eq!(
            XRoadError::AccessDenied("forbidden".into()).fault_code(),
            FaultCode::Client("Client.AccessDenied".into())
        );
        assert_eq!(
            XRoadError::ServiceNotFound("none".into()).fault_code(),
            FaultCode::Client("Client.ServiceNotFound".into())
        );
        assert_eq!(
            XRoadError::MessageProtocol("malformed".into()).fault_code(),
            FaultCode::Client("Client.InvalidMessage".into())
        );
        assert_eq!(
            XRoadError::Multipart("missing boundary".into()).fault_code(),
            FaultCode::Client("Client.InvalidMultipart".into())
        );
        assert_eq!(
            XRoadError::Asic("corrupt zip".into()).fault_code(),
            FaultCode::Client("Client.InvalidAsic".into())
        );
        assert_eq!(
            XRoadError::SignatureVerification("bad sig".into()).fault_code(),
            FaultCode::Server("Server.ServerProxy.InvalidSignature".into())
        );
        assert_eq!(
            XRoadError::Certificate("expired".into()).fault_code(),
            FaultCode::Server("Server.ServerProxy.InvalidCertificate".into())
        );
        assert_eq!(
            XRoadError::Ocsp("revoked".into()).fault_code(),
            FaultCode::Server("Server.ServerProxy.OcspError".into())
        );
        assert_eq!(
            XRoadError::Timeout.fault_code(),
            FaultCode::Server("Server.ServerProxy.Timeout".into())
        );
        assert_eq!(
            XRoadError::InternalFault {
                code: "Server.CustomFault".into(),
                message: "detail".into()
            }
            .fault_code(),
            FaultCode::Server("Server.CustomFault".into())
        );
        assert_eq!(
            XRoadError::Configuration("bad conf".into()).fault_code(),
            FaultCode::Server("Server.ServerProxy.InternalError".into())
        );
    }
}
