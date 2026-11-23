use common::{XRoadError, XRoadResult};
use xml_sec::c14n::{C14nAlgorithm, C14nMode, canonicalize_xml as xmlsec_c14n};

/// Canonicalizes an XML string according to W3C Canonical XML (C14N 1.0 / Exclusive C14N) rules:
/// 1. Document encoding is UTF-8 without BOM.
/// 2. Line breaks normalized to `\n` (0x0A).
/// 3. Attribute values and text normalized with standard entity escaping.
/// 4. Empty elements represented as `<tag></tag>` start-end tags (never `<tag/>`).
/// 5. XML declaration and comments stripped (without-comments canonicalization).
/// 6. Namespace declarations and attributes lexicographically sorted.
pub fn canonicalize_xml(xml: &str) -> XRoadResult<String> {
    let algo = C14nAlgorithm::new(C14nMode::Exclusive1_0, false);
    let bytes = xmlsec_c14n(xml.as_bytes(), &algo)
        .map_err(|e| XRoadError::Asic(format!("xml-sec C14N canonicalization error: {e}")))?;
    String::from_utf8(bytes)
        .map_err(|e| XRoadError::Asic(format!("Canonicalized XML is not valid UTF-8: {e}")))
}

/// Canonicalizes an XML string with inclusive C14N 1.0 mode.
pub fn canonicalize_xml_inclusive(xml: &str) -> XRoadResult<String> {
    let algo = C14nAlgorithm::new(C14nMode::Inclusive1_0, false);
    let bytes = xmlsec_c14n(xml.as_bytes(), &algo)
        .map_err(|e| XRoadError::Asic(format!("xml-sec C14N canonicalization error: {e}")))?;
    String::from_utf8(bytes)
        .map_err(|e| XRoadError::Asic(format!("Canonicalized XML is not valid UTF-8: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c14n_attribute_sorting() {
        let input =
            r#"<root z="last" a="first" xmlns:b="urn:b" xmlns="urn:default" xmlns:a="urn:a"/>"#;
        // Exclusive C14N strips unused namespace prefixes:
        let c14n_exc = canonicalize_xml(input).expect("C14N failed");
        assert_eq!(
            c14n_exc,
            r#"<root xmlns="urn:default" a="first" z="last"></root>"#
        );

        // Inclusive C14N retains all in-scope namespace declarations:
        let c14n_inc = canonicalize_xml_inclusive(input).expect("C14N failed");
        assert_eq!(
            c14n_inc,
            r#"<root xmlns="urn:default" xmlns:a="urn:a" xmlns:b="urn:b" a="first" z="last"></root>"#
        );
    }

    #[test]
    fn test_c14n_strips_declaration_and_comments() {
        let input = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!-- A comment -->\n<data>Hello &amp; World</data>";
        let c14n = canonicalize_xml(input).expect("C14N failed");
        assert_eq!(c14n, "<data>Hello &amp; World</data>");
    }

    #[test]
    fn test_c14n_empty_element_expansion() {
        let input = "<parent><child id=\"1\"/></parent>";
        let c14n = canonicalize_xml(input).expect("C14N failed");
        assert_eq!(c14n, "<parent><child id=\"1\"></child></parent>");
    }

    #[test]
    fn test_c14n_line_break_normalization() {
        let input = "<text>Line 1\r\nLine 2\rLine 3\nLine 4</text>";
        let c14n = canonicalize_xml(input).expect("C14N failed");
        assert_eq!(c14n, "<text>Line 1\nLine 2\nLine 3\nLine 4</text>");
    }

    #[test]
    fn test_xml_sec_c14n_roundtrip() {
        let input = "<root b=\"2\" a=\"1\"><empty/></root>";
        let s = canonicalize_xml(input).expect("xml-sec c14n failed");
        assert_eq!(s, "<root a=\"1\" b=\"2\"><empty></empty></root>");
    }
}
