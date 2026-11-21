use common::{XRoadError, XRoadResult};
use serde::{Deserialize, Serialize};

pub const MANIFEST_XMLNS: &str = "urn:oasis:names:tc:opendocument:xmlns:manifest:1.0";

/// XML DTO for the root `<manifest:manifest>` element.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename = "manifest:manifest")]
pub struct ManifestXml {
    #[serde(rename = "@xmlns:manifest")]
    pub xmlns_manifest: String,
    #[serde(rename = "manifest:file-entry", alias = "file-entry", default)]
    pub entries: Vec<ManifestFileEntryXml>,
}

/// XML DTO for `<manifest:file-entry>`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestFileEntryXml {
    #[serde(rename = "@manifest:full-path", alias = "@full-path")]
    pub full_path: String,
    #[serde(rename = "@manifest:media-type", alias = "@media-type")]
    pub media_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestFileEntry {
    pub full_path: String,
    pub media_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub xmlns_manifest: String,
    pub entries: Vec<ManifestFileEntry>,
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            xmlns_manifest: MANIFEST_XMLNS.to_string(),
            entries: vec![ManifestFileEntry {
                full_path: "/".to_string(),
                media_type: "application/vnd.etsi.asic-e+zip".to_string(),
            }],
        }
    }

    pub fn add_entry(&mut self, path: impl Into<String>, media_type: impl Into<String>) {
        self.entries.push(ManifestFileEntry {
            full_path: path.into(),
            media_type: media_type.into(),
        });
    }

    /// Serializes manifest into XML using `quick-xml` serde.
    pub fn to_xml(&self) -> XRoadResult<String> {
        let dto = ManifestXml {
            xmlns_manifest: self.xmlns_manifest.clone(),
            entries: self
                .entries
                .iter()
                .map(|e| ManifestFileEntryXml {
                    full_path: e.full_path.clone(),
                    media_type: e.media_type.clone(),
                })
                .collect(),
        };

        let mut buffer = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        let xml_str = quick_xml::se::to_string(&dto)
            .map_err(|e| XRoadError::Asic(format!("Failed to serialize manifest XML with quick-xml: {e}")))?;
        buffer.push_str(&xml_str);
        buffer.push('\n');
        Ok(buffer)
    }

    /// Deserializes manifest from XML using `quick-xml` serde.
    pub fn from_xml(xml: &str) -> XRoadResult<Self> {
        let dto: ManifestXml = quick_xml::de::from_str(xml)
            .map_err(|e| XRoadError::Asic(format!("Failed to deserialize manifest XML with quick-xml: {e}")))?;

        let entries = dto
            .entries
            .into_iter()
            .map(|e| ManifestFileEntry {
                full_path: e.full_path,
                media_type: e.media_type,
            })
            .collect();

        Ok(Self {
            xmlns_manifest: dto.xmlns_manifest,
            entries,
        })
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_serde() {
        let mut manifest = Manifest::new();
        manifest.add_entry("message.xml", "text/xml");
        manifest.add_entry("META-INF/signatures0.xml", "application/xades+xml");

        let xml = manifest.to_xml().unwrap();
        assert!(xml.contains("manifest:manifest"));
        assert!(xml.contains("message.xml"));

        let parsed = Manifest::from_xml(&xml).unwrap();
        assert_eq!(parsed.entries.len(), 3);
        assert_eq!(parsed.entries[1].full_path, "message.xml");
        assert_eq!(parsed.entries[1].media_type, "text/xml");
    }
}
