use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr};

/// Message protocol discriminator indicating SOAP or REST communication.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum MessageProtocol {
    Soap,
    Rest,
}

impl MessageProtocol {
    pub fn is_soap(&self) -> bool {
        matches!(self, Self::Soap)
    }

    pub fn is_rest(&self) -> bool {
        matches!(self, Self::Rest)
    }

    pub fn default_content_type(&self) -> &'static str {
        match self {
            Self::Soap => "text/xml; charset=utf-8",
            Self::Rest => "application/json",
        }
    }

    pub fn default_filename(&self) -> &'static str {
        match self {
            Self::Soap => "message.xml",
            Self::Rest => "message.json",
        }
    }
}

/// Backward-compatible alias for MessageProtocol.
pub type MessageDiscriminator = MessageProtocol;

/// Cryptographic token type (software keystore or hardware security module).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum TokenType {
    Software,
    #[strum(serialize = "HARDWARE", serialize = "HSM")]
    Hardware,
}

/// Purpose / usage of a cryptographic key or certificate.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum KeyUsage {
    Signing,
    Authentication,
}

/// Lifecycle status of an X-Road client.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
)]
#[strum(ascii_case_insensitive, serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientStatus {
    Saved,
    RegistrationInProgress,
    Registered,
    DeletionInProgress,
    GlobalError,
}

/// Lifecycle status of an X-Road certificate.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
)]
#[strum(ascii_case_insensitive, serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CertificateStatus {
    Saved,
    RegistrationInProgress,
    Registered,
    DeletionInProgress,
    Deleted,
    Revoked,
}

/// Supported service descriptions and protocols.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum ServiceType {
    Rest,
    Soap,
    Wsdl,
    OpenApi,
}

/// Types of central management requests.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
)]
#[strum(ascii_case_insensitive, serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManagementRequestType {
    AuthCertRegistration,
    AuthCertDeletion,
    ClientRegistration,
    ClientDeletion,
    OwnerChange,
}

/// Processing status of central management requests.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
)]
#[strum(ascii_case_insensitive, serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManagementRequestStatus {
    WaitingForApproval,
    Submitted,
    Approved,
    Declined,
    Revoked,
}

/// Originating source of central management requests.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
)]
#[strum(ascii_case_insensitive, serialize_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManagementRequestOrigin {
    SecurityServer,
    Center,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_message_protocol_strum_and_serde() {
        assert_eq!(MessageProtocol::Soap.to_string(), "SOAP");
        assert_eq!(MessageProtocol::Rest.to_string(), "REST");

        assert_eq!(
            MessageProtocol::from_str("soap").unwrap(),
            MessageProtocol::Soap
        );
        assert_eq!(
            MessageProtocol::from_str("SOAP").unwrap(),
            MessageProtocol::Soap
        );
        assert_eq!(
            MessageProtocol::from_str("rest").unwrap(),
            MessageProtocol::Rest
        );

        assert!(MessageProtocol::Soap.is_soap());
        assert!(!MessageProtocol::Soap.is_rest());
        assert_eq!(
            MessageProtocol::Soap.default_content_type(),
            "text/xml; charset=utf-8"
        );
        assert_eq!(MessageProtocol::Soap.default_filename(), "message.xml");

        let json = serde_json::to_string(&MessageProtocol::Soap).unwrap();
        assert_eq!(json, r#""SOAP""#);
        let parsed: MessageProtocol = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, MessageProtocol::Soap);
    }

    #[test]
    fn test_token_type_strum() {
        assert_eq!(TokenType::Software.to_string(), "SOFTWARE");
        assert_eq!(
            TokenType::from_str("software").unwrap(),
            TokenType::Software
        );
        assert_eq!(TokenType::from_str("hsm").unwrap(), TokenType::Hardware);
        assert_eq!(
            TokenType::from_str("hardware").unwrap(),
            TokenType::Hardware
        );
    }

    #[test]
    fn test_client_and_management_statuses() {
        assert_eq!(ClientStatus::Registered.to_string(), "REGISTERED");
        assert_eq!(
            ClientStatus::from_str("registered").unwrap(),
            ClientStatus::Registered
        );

        assert_eq!(
            ManagementRequestStatus::WaitingForApproval.to_string(),
            "WAITING_FOR_APPROVAL"
        );
        assert_eq!(
            ManagementRequestStatus::from_str("waiting_for_approval").unwrap(),
            ManagementRequestStatus::WaitingForApproval
        );

        assert_eq!(
            ManagementRequestType::ClientRegistration.to_string(),
            "CLIENT_REGISTRATION"
        );
        assert_eq!(
            ManagementRequestType::from_str("client_registration").unwrap(),
            ManagementRequestType::ClientRegistration
        );
    }
}
