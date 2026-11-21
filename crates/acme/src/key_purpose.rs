use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr};

/// The purpose a key/certificate ordered through ACME is used for in X-Road.
///
/// Conforms to X-Road's upstream `AcmeKeyPurpose`.
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
pub enum AcmeKeyPurpose {
    Authentication,
    Signing,
}
