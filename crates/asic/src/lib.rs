use common::{XRoadError, XRoadResult};
use std::io::{Cursor, Read, Write};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

pub const ASIC_MIME_TYPE: &str = "application/vnd.etsi.asic-e+zip";

/// ASiC-e (Associated Signature Container Extended) builder and parser.
pub struct AsicContainer {
    pub message_data: Vec<u8>,
    pub signature_data: Vec<u8>,
    pub timestamp_data: Option<Vec<u8>>,
}

impl AsicContainer {
    pub fn new(message: Vec<u8>, signature: Vec<u8>) -> Self {
        Self {
            message_data: message,
            signature_data: signature,
            timestamp_data: None,
        }
    }

    /// Packs the container into raw zip bytes conforming to ETSI TS 102 918.
    pub fn pack(&self) -> XRoadResult<Vec<u8>> {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
            let options = SimpleFileOptions::default();

            // 1. mimetype (first uncompressed file)
            zip.start_file("mimetype", options)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            zip.write_all(ASIC_MIME_TYPE.as_bytes())
                .map_err(|e| XRoadError::Asic(e.to_string()))?;

            // 2. message payload
            zip.start_file("message.xml", options)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            zip.write_all(&self.message_data)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;

            // 3. signature
            zip.start_file("META-INF/signatures0.xml", options)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            zip.write_all(&self.signature_data)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;

            // 4. optional timestamp
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

    /// Unpacks an ASiC-e container from raw zip bytes.
    pub fn unpack(bytes: &[u8]) -> XRoadResult<Self> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor).map_err(|e| XRoadError::Asic(e.to_string()))?;

        let mut message_data = Vec::new();
        let mut signature_data = Vec::new();
        let mut timestamp_data = None;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            match file.name() {
                "message.xml" => {
                    file.read_to_end(&mut message_data)
                        .map_err(|e| XRoadError::Asic(e.to_string()))?;
                }
                "META-INF/signatures0.xml" => {
                    file.read_to_end(&mut signature_data)
                        .map_err(|e| XRoadError::Asic(e.to_string()))?;
                }
                "META-INF/timestamp.tsr" => {
                    let mut ts = Vec::new();
                    file.read_to_end(&mut ts)
                        .map_err(|e| XRoadError::Asic(e.to_string()))?;
                    timestamp_data = Some(ts);
                }
                _ => {}
            }
        }

        Ok(Self {
            message_data,
            signature_data,
            timestamp_data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asic_pack_unpack() {
        let container = AsicContainer::new(b"<request/>".to_vec(), b"<signature/>".to_vec());
        let packed = container.pack().unwrap();
        assert!(!packed.is_empty());

        let unpacked = AsicContainer::unpack(&packed).unwrap();
        assert_eq!(unpacked.message_data, b"<request/>");
        assert_eq!(unpacked.signature_data, b"<signature/>");
    }
}
