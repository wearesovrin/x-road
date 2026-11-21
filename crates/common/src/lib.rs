pub mod error;
pub mod time;

pub use error::{FaultCode, XRoadError, XRoadResult};
pub use time::CallBudget;

/// Default X-Road protocol version.
pub const XROAD_PROTOCOL_VERSION: &str = "4.0";

/// Standard X-Road HTTP headers.
pub mod headers {
    pub const X_ROAD_CLIENT: &str = "x-road-client";
    pub const X_ROAD_SERVICE: &str = "x-road-service";
    pub const X_ROAD_ID: &str = "x-road-id";
    pub const X_ROAD_USER_ID: &str = "x-road-userid";
    pub const X_ROAD_ISSUE: &str = "x-road-issue";
    pub const X_ROAD_PROTOCOL_VERSION: &str = "x-road-protocol-version";
    pub const X_ROAD_REPRESENTED_PARTY: &str = "x-road-represented-party";
    pub const X_ROAD_REQUEST_ID: &str = "x-road-request-id";
    pub const X_ROAD_ORIGINAL_CONTENT_TYPE: &str = "x-original-content-type";
    pub const X_HASH_ALGORITHM: &str = "x-hash-algorithm";
}
