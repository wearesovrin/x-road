use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr};

/// X-Road Object Type indicating the hierarchy of an identifier.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    EnumString,
    AsRefStr,
    IntoStaticStr,
    Serialize,
    Deserialize,
)]
#[strum(ascii_case_insensitive, serialize_all = "UPPERCASE")]
#[serde(rename_all = "UPPERCASE")]
pub enum XRoadObjectType {
    Server,
    Service,
    Member,
    Subsystem,
    GlobalGroup,
    LocalGroup,
    CentralService,
}

impl XRoadObjectType {
    /// Returns the uppercase identifier representation conforming to X-Road XML/spec standard.
    #[inline]
    pub fn as_str(&self) -> &'static str {
        self.into()
    }
}
