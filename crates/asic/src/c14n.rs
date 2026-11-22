use common::{XRoadError, XRoadResult};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

/// Canonicalizes an XML string according to W3C Canonical XML (C14N 1.0 / Exclusive C14N) rules:
/// 1. Document encoding is UTF-8 without BOM.
/// 2. Line breaks normalized to `\n` (0x0A).
/// 3. Attribute values and text normalized with standard entity escaping.
/// 4. Empty elements represented as `<tag></tag>` start-end tags (never `<tag/>`).
/// 5. XML declaration and comments stripped (without-comments canonicalization).
/// 6. Namespace declarations and attributes lexicographically sorted.
pub fn canonicalize_xml(xml: &str) -> XRoadResult<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut out = String::with_capacity(xml.len());
    let mut buf = Vec::new();
    let mut depth: usize = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                depth += 1;
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                out.push('<');
                out.push_str(&tag);
                serialize_sorted_attributes(&e, &mut out)?;
                out.push('>');
            }
            Ok(Event::End(e)) => {
                if depth > 0 {
                    depth -= 1;
                }
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                out.push_str("</");
                out.push_str(&tag);
                out.push('>');
            }
            Ok(Event::Empty(e)) => {
                // C14N specifies that empty elements MUST be expanded to <elem></elem>
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                out.push('<');
                out.push_str(&tag);
                serialize_sorted_attributes(&e, &mut out)?;
                out.push('>');
                out.push_str("</");
                out.push_str(&tag);
                out.push('>');
            }
            Ok(Event::Text(e)) => {
                // Per W3C C14N 1.0 section 4.1: whitespace outside document element is discarded.
                if depth > 0 {
                    let text = e.unescape().map_err(|err| {
                        XRoadError::Asic(format!("Failed to unescape text: {err}"))
                    })?;
                    escape_xml_text(&text, &mut out);
                }
            }
            Ok(Event::CData(e)) => {
                if depth > 0 {
                    // In C14N, CDATA sections are replaced by their character content with entity escaping
                    let text = String::from_utf8_lossy(e.as_ref());
                    escape_xml_text(&text, &mut out);
                }
            }
            Ok(Event::Decl(_)) | Ok(Event::DocType(_)) | Ok(Event::Comment(_)) => {
                // Stripped in C14N without comments
            }
            Ok(Event::PI(e)) => {
                if depth > 0 {
                    let pi = String::from_utf8_lossy(e.as_ref());
                    out.push_str("<?");
                    out.push_str(&pi);
                    out.push_str("?>");
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(XRoadError::Asic(format!(
                    "XML parsing error during C14N canonicalization: {e}"
                )));
            }
        }
        buf.clear();
    }

    Ok(out)
}

/// Serializes element attributes with C14N sorting:
/// - Namespace declarations (`xmlns` and `xmlns:*`) come first.
/// - Default namespace `xmlns` precedes prefixed namespaces.
/// - Prefixed namespaces sorted lexicographically by prefix.
/// - Normal attributes sorted lexicographically by qualified name.
fn serialize_sorted_attributes(
    e: &quick_xml::events::BytesStart,
    out: &mut String,
) -> XRoadResult<()> {
    let mut ns_decls: Vec<(String, String)> = Vec::new();
    let mut attrs: Vec<(String, String)> = Vec::new();

    for attr_res in e.attributes() {
        let attr =
            attr_res.map_err(|err| XRoadError::Asic(format!("Invalid attribute in XML: {err}")))?;
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = attr
            .unescape_value()
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| String::from_utf8_lossy(&attr.value).into_owned());

        if key == "xmlns" {
            ns_decls.push(("".to_string(), val));
        } else if let Some(prefix) = key.strip_prefix("xmlns:") {
            ns_decls.push((prefix.to_string(), val));
        } else {
            attrs.push((key, val));
        }
    }

    // Sort namespaces: default first, then alphabetical by prefix
    ns_decls.sort_by(|a, b| match (a.0.is_empty(), b.0.is_empty()) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.0.cmp(&b.0),
    });

    // Sort attributes lexicographically by key name
    attrs.sort_by(|a, b| a.0.cmp(&b.0));

    // Emit namespaces
    for (prefix, val) in ns_decls {
        out.push(' ');
        if prefix.is_empty() {
            out.push_str("xmlns=\"");
        } else {
            out.push_str("xmlns:");
            out.push_str(&prefix);
            out.push_str("=\"");
        }
        escape_xml_attr(&val, out);
        out.push('"');
    }

    // Emit normal attributes
    for (key, val) in attrs {
        out.push(' ');
        out.push_str(&key);
        out.push_str("=\"");
        escape_xml_attr(&val, out);
        out.push('"');
    }

    Ok(())
}

/// Escapes XML character content according to W3C C14N:
/// `&` -> `&amp;`, `<` -> `&lt;`, `>` -> `&gt;`, `\r` -> `&#xD;`.
/// Also normalizes line endings (`\r\n` -> `\n`).
fn escape_xml_text(text: &str, out: &mut String) {
    let normalized = text.replace("\r\n", "\n");
    for ch in normalized.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\r' => out.push_str("&#xD;"),
            _ => out.push(ch),
        }
    }
}

/// Escapes attribute values according to W3C C14N:
/// `&` -> `&amp;`, `<` -> `&lt;`, `"` -> `&quot;`, `\t` -> `&#x9;`, `\n` -> `&#xA;`, `\r` -> `&#xD;`.
fn escape_xml_attr(val: &str, out: &mut String) {
    for ch in val.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '"' => out.push_str("&quot;"),
            '\t' => out.push_str("&#x9;"),
            '\n' => out.push_str("&#xA;"),
            '\r' => out.push_str("&#xD;"),
            _ => out.push(ch),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c14n_attribute_sorting() {
        let input =
            r#"<root z="last" a="first" xmlns:b="urn:b" xmlns="urn:default" xmlns:a="urn:a"/>"#;
        let c14n = canonicalize_xml(input).expect("C14N failed");
        assert_eq!(
            c14n,
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
        assert_eq!(c14n, "<text>Line 1\nLine 2&#xD;Line 3\nLine 4</text>");
    }
}
