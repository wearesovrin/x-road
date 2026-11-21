use crate::object_type::XRoadObjectType;

/// Base trait for X-Road identifiers.
pub trait XRoadId: std::fmt::Display + std::fmt::Debug {
    /// Returns the object type of this identifier.
    fn object_type(&self) -> XRoadObjectType;

    /// Returns the X-Road instance code, if applicable.
    fn xroad_instance(&self) -> Option<&str>;

    /// Returns fields for string formatting (excluding instance and object type).
    fn fields_for_string_format(&self) -> Vec<&str>;

    /// Formats as a short string with the given delimiter (default: '/').
    fn to_short_string(&self, delimiter: char) -> String {
        let mut parts = Vec::new();
        if let Some(instance) = self.xroad_instance() {
            parts.push(instance);
        }
        for field in self.fields_for_string_format() {
            parts.push(field);
        }
        parts.join(&delimiter.to_string())
    }

    /// Formats as an encoded identifier string.
    ///
    /// When `include_type` is true: `<TYPE>:<INSTANCE>/<FIELD1>/...`
    /// When `include_type` is false: `<INSTANCE>/<FIELD1>/...`
    fn to_encoded_id(&self, include_type: bool) -> String {
        let short = self.to_short_string('/');
        if include_type {
            format!("{}:{}", self.object_type(), short)
        } else {
            short
        }
    }
}
