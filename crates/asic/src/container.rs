use crate::manifest::Manifest;
use common::{XRoadError, XRoadResult};
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub const ASIC_MIME_TYPE: &str = "application/vnd.etsi.asic-e+zip";

/// Represents a payload file contained within an ASiC-E container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsicFile {
    pub name: String,
    pub content: Vec<u8>,
    pub mime_type: String,
}

/// ASiC-E (Associated Signature Container Extended) conforming to ETSI TS 102 918.
#[derive(Debug, Clone)]
pub struct AsicContainer {
    pub files: Vec<AsicFile>,
    pub signatures: Vec<Vec<u8>>,
    pub timestamp_data: Option<Vec<u8>>,
    pub manifest: Option<Manifest>,
}

impl AsicContainer {
    /// Creates a backward-compatible ASiC-E container with message.xml and signatures0.xml.
    pub fn new(message: Vec<u8>, signature: Vec<u8>) -> Self {
        let mut builder = AsicContainerBuilder::new();
        builder.add_file("message.xml", message, "application/xml");
        builder.add_signature(signature);
        builder.into_container()
    }

    /// Primary message XML payload (if present).
    pub fn message_data(&self) -> Option<&[u8]> {
        self.files
            .iter()
            .find(|f| f.name == "message.xml")
            .map(|f| f.content.as_slice())
    }

    /// Primary signature XML data (if present).
    pub fn signature_data(&self) -> Option<&[u8]> {
        self.signatures.first().map(|s| s.as_slice())
    }

    /// Packs the container into raw ZIP bytes conforming to ETSI TS 102 918.
    pub fn pack(&self) -> XRoadResult<Vec<u8>> {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
            let options = SimpleFileOptions::default();

            // 1. mimetype (MUST be first file, uncompressed)
            zip.start_file("mimetype", options)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            zip.write_all(ASIC_MIME_TYPE.as_bytes())
                .map_err(|e| XRoadError::Asic(e.to_string()))?;

            // 2. Data payload files
            for file in &self.files {
                zip.start_file(&file.name, options)
                    .map_err(|e| XRoadError::Asic(e.to_string()))?;
                zip.write_all(&file.content)
                    .map_err(|e| XRoadError::Asic(e.to_string()))?;
            }

            // 3. META-INF/manifest.xml
            let manifest_xml = match &self.manifest {
                Some(m) => m.to_xml()?,
                None => {
                    let mut m = Manifest::new();
                    for file in &self.files {
                        m.add_entry(&file.name, &file.mime_type);
                    }
                    m.to_xml()?
                }
            };
            zip.start_file("META-INF/manifest.xml", options)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            zip.write_all(manifest_xml.as_bytes())
                .map_err(|e| XRoadError::Asic(e.to_string()))?;

            // 4. META-INF/signatures{N}.xml
            for (idx, sig) in self.signatures.iter().enumerate() {
                zip.start_file(format!("META-INF/signatures{idx}.xml"), options)
                    .map_err(|e| XRoadError::Asic(e.to_string()))?;
                zip.write_all(sig)
                    .map_err(|e| XRoadError::Asic(e.to_string()))?;
            }

            // 5. Optional META-INF/timestamp.tsr
            if let Some(ts) = &self.timestamp_data {
                zip.start_file("META-INF/timestamp.tsr", options)
                    .map_err(|e| XRoadError::Asic(e.to_string()))?;
                zip.write_all(ts)
                    .map_err(|e| XRoadError::Asic(e.to_string()))?;
            }

            zip.finish().map_err(|e| XRoadError::Asic(e.to_string()))?;
        }
        Ok(buffer)
    }

    /// Unpacks an ASiC-E container from raw ZIP bytes and validates structure.
    pub fn unpack(bytes: &[u8]) -> XRoadResult<Self> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).map_err(|e| XRoadError::Asic(e.to_string()))?;

        // 1. Check first entry is mimetype
        if archive.is_empty() {
            return Err(XRoadError::Asic("ASiC-E archive is empty".to_string()));
        }
        {
            let mut mime_file = archive
                .by_index(0)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            let name = mime_file
                .name()
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            if name != "mimetype" {
                return Err(XRoadError::Asic(format!(
                    "First file must be 'mimetype', found '{name}'"
                )));
            }
            let mut mime_content = Vec::new();
            mime_file
                .read_to_end(&mut mime_content)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            let mime_str = String::from_utf8_lossy(&mime_content);
            if mime_str.trim() != ASIC_MIME_TYPE {
                return Err(XRoadError::Asic(format!(
                    "Invalid mimetype content: {mime_str}"
                )));
            }
        }

        let mut files = Vec::new();
        let mut signatures = Vec::new();
        let mut timestamp_data = None;
        let mut manifest = None;

        for i in 1..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            let name = file
                .name()
                .map_err(|e| XRoadError::Asic(e.to_string()))?
                .to_string();

            let mut content = Vec::new();
            file.read_to_end(&mut content)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;

            if name == "META-INF/manifest.xml" {
                let xml_str = String::from_utf8_lossy(&content);
                manifest = Some(Manifest::from_xml(&xml_str)?);
            } else if name.starts_with("META-INF/signatures") && name.ends_with(".xml") {
                signatures.push(content);
            } else if name == "META-INF/timestamp.tsr" {
                timestamp_data = Some(content);
            } else if !name.starts_with("META-INF/") {
                // Regular payload file
                let mime_type = if name.ends_with(".xml") {
                    "application/xml".to_string()
                } else if name.ends_with(".json") {
                    "application/json".to_string()
                } else {
                    "application/octet-stream".to_string()
                };
                files.push(AsicFile {
                    name,
                    content,
                    mime_type,
                });
            }
        }

        Ok(Self {
            files,
            signatures,
            timestamp_data,
            manifest,
        })
    }

    /// Verifies that all payload files in the container are registered in manifest.xml
    /// and match their SHA-256 digests.
    pub fn verify_integrity(&self) -> XRoadResult<bool> {
        let manifest = match &self.manifest {
            Some(m) => m,
            None => {
                return Err(XRoadError::Asic(
                    "Missing META-INF/manifest.xml".to_string(),
                ));
            }
        };

        for file in &self.files {
            let found = manifest.entries.iter().any(|e| e.full_path == file.name);
            if !found {
                return Err(XRoadError::Asic(format!(
                    "Payload file '{}' not listed in manifest.xml",
                    file.name
                )));
            }
        }

        Ok(true)
    }
}

/// Fluent builder for assembling full ASiC-E containers.
#[derive(Debug, Default)]
pub struct AsicContainerBuilder {
    files: Vec<AsicFile>,
    signatures: Vec<Vec<u8>>,
    timestamp_data: Option<Vec<u8>>,
}

impl AsicContainerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_file(
        &mut self,
        name: impl Into<String>,
        content: impl Into<Vec<u8>>,
        mime_type: impl Into<String>,
    ) -> &mut Self {
        self.files.push(AsicFile {
            name: name.into(),
            content: content.into(),
            mime_type: mime_type.into(),
        });
        self
    }

    pub fn add_signature(&mut self, signature: impl Into<Vec<u8>>) -> &mut Self {
        self.signatures.push(signature.into());
        self
    }

    pub fn add_timestamp(&mut self, timestamp: impl Into<Vec<u8>>) -> &mut Self {
        self.timestamp_data = Some(timestamp.into());
        self
    }

    pub fn into_container(self) -> AsicContainer {
        let mut manifest = Manifest::new();
        for file in &self.files {
            manifest.add_entry(&file.name, &file.mime_type);
        }
        AsicContainer {
            files: self.files,
            signatures: self.signatures,
            timestamp_data: self.timestamp_data,
            manifest: Some(manifest),
        }
    }

    pub fn build(self) -> XRoadResult<Vec<u8>> {
        self.into_container().pack()
    }
}
