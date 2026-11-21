use common::{XRoadError, XRoadResult};
use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub const MANIFEST_XMLNS: &str = "urn:oasis:names:tc:opendocument:xmlns:manifest:1.0";

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

    pub fn to_xml(&self) -> XRoadResult<String> {
        let mut buffer = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        buffer.push_str("<manifest:manifest xmlns:manifest=\"urn:oasis:names:tc:opendocument:xmlns:manifest:1.0\">\n");
        for entry in &self.entries {
            buffer.push_str(&format!(
                "  <manifest:file-entry manifest:full-path=\"{}\" manifest:media-type=\"{}\"/>\n",
                entry.full_path, entry.media_type
            ));
        }
        buffer.push_str("</manifest:manifest>\n");
        Ok(buffer)
    }

    pub fn from_xml(xml: &str) -> XRoadResult<Self> {
        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);

        let mut entries = Vec::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                    let local_name = e.local_name();
                    if local_name.as_ref() == b"file-entry" {
                        let mut full_path = String::new();
                        let mut media_type = String::new();

                        for attr in e.attributes().flatten() {
                            let attr_local = attr.key.local_name();
                            if attr_local.as_ref() == b"full-path" {
                                full_path = String::from_utf8_lossy(&attr.value).to_string();
                            } else if attr_local.as_ref() == b"media-type" {
                                media_type = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }

                        if !full_path.is_empty() {
                            entries.push(ManifestFileEntry {
                                full_path,
                                media_type,
                            });
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(XRoadError::Asic(format!("XML error parsing manifest: {e}"))),
                _ => {}
            }
            buf.clear();
        }

        Ok(Self {
            xmlns_manifest: MANIFEST_XMLNS.to_string(),
            entries,
        })
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}
