use common::{XRoadError, XRoadResult};

/// Maximum allowed character length for any identifier component.
pub const MAX_IDENTIFIER_PART_LENGTH: usize = 255;

/// Strict character validation matching X-Road's `StrictIdentifierValidator`.
///
/// In X-Road, identifier parts (instances, member classes, member codes, subsystem codes,
/// service codes, and server codes) are restricted to ASCII alphanumeric characters
/// and the special punctuation symbols `' ( ) + , - . = ?`.
pub struct StrictIdentifierValidator;

impl StrictIdentifierValidator {
    /// Determines whether the given character is allowed in an X-Road identifier.
    #[inline]
    pub fn is_valid_char(c: char) -> bool {
        c.is_ascii_alphanumeric()
            || matches!(c, '\'' | '(' | ')' | '+' | ',' | '-' | '.' | '=' | '?')
    }

    /// Determines whether the string contains only valid characters and does not exceed the maximum length.
    pub fn is_valid(s: &str) -> bool {
        if s.is_empty() || s.len() > MAX_IDENTIFIER_PART_LENGTH {
            return false;
        }
        s.chars().all(Self::is_valid_char)
    }

    /// Validates a mandatory identifier component.
    pub fn validate_part(field_name: &str, value: &str) -> XRoadResult<()> {
        if value.trim().is_empty() {
            return Err(XRoadError::IdentifierValidation(format!(
                "Field '{}' cannot be empty or blank",
                field_name
            )));
        }

        if value.len() > MAX_IDENTIFIER_PART_LENGTH {
            return Err(XRoadError::IdentifierValidation(format!(
                "Field '{}' exceeds maximum allowed length of {} characters (actual: {})",
                field_name,
                MAX_IDENTIFIER_PART_LENGTH,
                value.len()
            )));
        }

        for (idx, ch) in value.char_indices() {
            if !Self::is_valid_char(ch) {
                return Err(XRoadError::IdentifierValidation(format!(
                    "Field '{}' contains invalid character '{}' (0x{:02X}) at position {}",
                    field_name, ch, ch as u32, idx
                )));
            }
        }

        Ok(())
    }

    /// Validates an optional identifier component (e.g. subsystem code, service version).
    pub fn validate_optional_part(field_name: &str, value: Option<&str>) -> XRoadResult<()> {
        if let Some(val) = value {
            Self::validate_part(field_name, val)?;
        }
        Ok(())
    }
}
