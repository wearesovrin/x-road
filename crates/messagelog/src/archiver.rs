use asic::{AsicContainer, AsicContainerBuilder};
use chrono::Utc;
use common::{XRoadError, XRoadResult};
use sha2::{Digest, Sha512};
use std::fs::{self, File};
use std::io::{Cursor, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::services::MessageLogDb;

/// Helper function to encode raw bytes into lowercase hexadecimal string.
fn hex_encode(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut v = Vec::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        v.push(HEX_CHARS[(byte >> 4) as usize]);
        v.push(HEX_CHARS[(byte & 0x0f) as usize]);
    }
    unsafe { String::from_utf8_unchecked(v) }
}

/// Computes SHA-512 hexadecimal digest of the given bytes.
fn sha512_hex(data: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hex_encode(&hasher.finalize())
}

/// Chained digest calculation according to X-Road specification:
/// `combined = prev_hex + current_file_hex`
/// `result = sha512_hex(combined.as_bytes())`
fn chain_digest(prev_hex: &str, file_bytes: &[u8]) -> String {
    let file_hex = sha512_hex(file_bytes);
    let combined = format!("{prev_hex}{file_hex}");
    sha512_hex(combined.as_bytes())
}

/// Metadata and status returned upon a successful archival run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveInfo {
    pub file_path: PathBuf,
    pub file_name: String,
    pub record_count: usize,
    pub last_digest: String,
}

/// Results returned from verifying an X-Road message log archive.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArchiveVerificationResult {
    pub file_path: String,
    pub records_verified: usize,
    pub containers: Vec<String>,
    pub prev_digest: String,
    pub last_digest: String,
    pub valid: bool,
}

/// Archiver responsible for reading unarchived message logs from the database,
/// packaging them into ETSI TS 102 918 ASiC-E containers, computing the cryptographic
/// linking hash chain, and saving the final archive `.zip` file.
pub struct MessageLogArchiver {
    db: MessageLogDb,
    archive_dir: PathBuf,
    max_records: usize,
}

impl MessageLogArchiver {
    pub fn new(db: MessageLogDb, archive_dir: impl Into<PathBuf>) -> Self {
        Self {
            db,
            archive_dir: archive_dir.into(),
            max_records: 1000,
        }
    }

    pub fn with_max_records(mut self, max: usize) -> Self {
        self.max_records = max;
        self
    }

    /// Archives a batch of pending unarchived records.
    ///
    /// Returns `Ok(Some(ArchiveInfo))` if records were archived, or `Ok(None)` if no
    /// unarchived records were available.
    pub async fn archive_batch(&self) -> XRoadResult<Option<ArchiveInfo>> {
        let records = self.db.find_unarchived(self.max_records as u64).await?;
        if records.is_empty() {
            return Ok(None);
        }

        // Ensure archive directory exists
        if !self.archive_dir.exists() {
            fs::create_dir_all(&self.archive_dir).map_err(|e| XRoadError::Io(e))?;
        }

        let mut archive_buffer = Vec::new();
        let mut zip = ZipWriter::new(Cursor::new(&mut archive_buffer));
        let options = SimpleFileOptions::default();

        let mut current_digest = String::new();
        let mut linking_lines = Vec::new();

        // Header line: "<prevDigest> <prevArchFile> SHA-512"
        linking_lines.push("- - SHA-512".to_string());

        let mut archived_ids = Vec::with_capacity(records.len());

        for record in &records {
            archived_ids.push(record.id);

            // Fetch attachments for this record if any exist
            let attachments = self.db.get_attachments(record.id).await?;

            // Build ASiC container for this message record
            let mut builder = AsicContainerBuilder::new();

            let (msg_filename, mime_type) = if record.protocol().is_rest()
                || record
                    .message_body
                    .as_deref()
                    .unwrap_or("")
                    .trim_start()
                    .starts_with('{')
                || record
                    .message_body
                    .as_deref()
                    .unwrap_or("")
                    .trim_start()
                    .starts_with('[')
            {
                ("message.json", "application/json")
            } else {
                ("message.xml", "application/xml")
            };

            let body_bytes = record.message_body.clone().unwrap_or_default().into_bytes();
            builder.add_file(msg_filename, body_bytes, mime_type);

            if let Some(ref sig_data) = record.signature_data {
                builder.add_signature(sig_data.clone());
            }

            for att in attachments {
                let att_name = format!("attachment_{}.bin", att.attachment_no);
                let att_bytes = att.data.unwrap_or_default();
                builder.add_file(att_name, att_bytes, "application/octet-stream");
            }

            let container_bytes = builder.build()?;

            // Construct container filename: "{clean_query_id}-{type}-{id_hex}.asice"
            let clean_query_id: String = record
                .query_id
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
            let record_type = if record.response {
                "response"
            } else {
                "request"
            };
            let container_filename =
                format!("{clean_query_id}-{record_type}-{:x}.asice", record.id);

            // Chain digest
            current_digest = chain_digest(&current_digest, &container_bytes);
            linking_lines.push(format!("{current_digest} {container_filename}"));

            // Write container to archive zip
            zip.start_file(&container_filename, options)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
            zip.write_all(&container_bytes)
                .map_err(|e| XRoadError::Asic(e.to_string()))?;
        }

        // Add linkinginfo to zip root
        let linking_info_content = linking_lines.join("\n") + "\n";
        zip.start_file("linkinginfo", options)
            .map_err(|e| XRoadError::Asic(e.to_string()))?;
        zip.write_all(linking_info_content.as_bytes())
            .map_err(|e| XRoadError::Asic(e.to_string()))?;

        zip.finish().map_err(|e| XRoadError::Asic(e.to_string()))?;

        // Format archive filename: "mlog-{start}-{end}-{digest_prefix}.zip"
        let start_time = records.first().map(|r| r.time).unwrap_or_else(Utc::now);
        let end_time = records.last().map(|r| r.time).unwrap_or_else(Utc::now);
        let digest_prefix = &current_digest[..16.min(current_digest.len())];
        let filename = format!(
            "mlog-{}-{}-{digest_prefix}.zip",
            start_time.format("%Y%m%d%H%M%S"),
            end_time.format("%Y%m%d%H%M%S")
        );

        let final_path = self.archive_dir.join(&filename);
        let tmp_path = self.archive_dir.join(format!(".tmp-{filename}"));

        fs::write(&tmp_path, &archive_buffer).map_err(|e| XRoadError::Io(e))?;
        fs::rename(&tmp_path, &final_path).map_err(|e| XRoadError::Io(e))?;

        // Mark processed records as archived in the database
        self.db.mark_archived(&archived_ids).await?;

        Ok(Some(ArchiveInfo {
            file_path: final_path,
            file_name: filename,
            record_count: records.len(),
            last_digest: current_digest,
        }))
    }
}

/// Verifies an X-Road message log archive file.
///
/// Ensures:
/// 1. The file is a valid ZIP archive containing `linkinginfo`.
/// 2. The digest algorithm is supported (SHA-512).
/// 3. If a `prev_digest_override` is given, it matches the archive header's previous digest.
/// 4. Every `.asice` entry listed in `linkinginfo` matches the cryptographic chain digest.
/// 5. Every extracted `.asice` entry is a valid ASiC-E container with verified internal integrity.
pub fn verify_archive(
    path: impl AsRef<Path>,
    prev_digest_override: Option<&str>,
) -> XRoadResult<ArchiveVerificationResult> {
    let path_ref = path.as_ref();
    let file = File::open(path_ref).map_err(|e| XRoadError::Io(e))?;
    let mut zip = ZipArchive::new(file).map_err(|e| XRoadError::Asic(e.to_string()))?;

    // 1. Read linkinginfo
    let mut linking_entry = zip
        .by_name("linkinginfo")
        .map_err(|_| XRoadError::Asic("Missing 'linkinginfo' file in archive".to_string()))?;

    let mut linking_content = String::new();
    linking_entry
        .read_to_string(&mut linking_content)
        .map_err(|e| XRoadError::Io(e))?;
    drop(linking_entry);

    let lines: Vec<&str> = linking_content.lines().collect();
    if lines.is_empty() {
        return Err(XRoadError::Asic("Empty linkinginfo in archive".to_string()));
    }

    // Parse header line: "<prevDigest> <prevArchFile> SHA-512"
    let header_parts: Vec<&str> = lines[0].split_whitespace().collect();
    if header_parts.len() < 3 {
        return Err(XRoadError::Asic(format!(
            "Invalid linkinginfo header line: '{}'",
            lines[0]
        )));
    }

    let raw_prev_digest = header_parts[0];
    let prev_digest = if raw_prev_digest == "-" {
        String::new()
    } else {
        raw_prev_digest.to_string()
    };
    let algo = header_parts[2];

    if algo != "SHA-512" {
        return Err(XRoadError::Asic(format!(
            "Unsupported archive digest algorithm: '{algo}', expected SHA-512"
        )));
    }

    if let Some(expected_prev) = prev_digest_override {
        let normalized_expected = if expected_prev == "-" {
            ""
        } else {
            expected_prev
        };
        if prev_digest != normalized_expected {
            return Err(XRoadError::Asic(format!(
                "Previous digest mismatch: archive specified '{prev_digest}', caller expected '{normalized_expected}'"
            )));
        }
    }

    let mut current_digest = prev_digest.clone();
    let mut verified_containers = Vec::new();

    // Verify each container in sequence
    for (idx, line) in lines.iter().skip(1).enumerate() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            return Err(XRoadError::Asic(format!(
                "Invalid linkinginfo entry at line {}: '{}'",
                idx + 2,
                line
            )));
        }

        let recorded_digest = parts[0];
        let filename = parts[1];

        // Read container from ZIP
        let mut entry = zip.by_name(filename).map_err(|e| {
            XRoadError::Asic(format!(
                "Archive missing file '{filename}' specified in linkinginfo: {e}"
            ))
        })?;

        let mut container_bytes = Vec::new();
        entry
            .read_to_end(&mut container_bytes)
            .map_err(|e| XRoadError::Io(e))?;
        drop(entry);

        // Verify cryptographic hash chain
        current_digest = chain_digest(&current_digest, &container_bytes);
        if current_digest != recorded_digest {
            return Err(XRoadError::Asic(format!(
                "Hash chain digest mismatch for '{filename}': computed '{current_digest}', recorded '{recorded_digest}'"
            )));
        }

        // Validate ASiC container internal integrity
        let asic = AsicContainer::unpack(&container_bytes).map_err(|e| {
            XRoadError::Asic(format!("Failed unpacking ASiC container '{filename}': {e}"))
        })?;
        asic.verify_integrity().map_err(|e| {
            XRoadError::Asic(format!("Integrity check failed for '{filename}': {e}"))
        })?;

        verified_containers.push(filename.to_string());
    }

    Ok(ArchiveVerificationResult {
        file_path: path_ref.to_string_lossy().to_string(),
        records_verified: verified_containers.len(),
        containers: verified_containers,
        prev_digest,
        last_digest: current_digest,
        valid: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::NewLogRecord;

    #[tokio::test]
    async fn test_archiver_batch_and_verification() {
        let db = MessageLogDb::connect_in_memory()
            .await
            .expect("Failed to connect messagelog DB");

        // Insert sample records
        let id1 = db
            .log_message(NewLogRecord {
                query_id: "query-001".to_string(),
                x_request_id: Some("req-1".to_string()),
                client_id_str: Some("DEV/COM/123/CLIENT".to_string()),
                service_id_str: Some("DEV/COM/123/SERVICE".to_string()),
                discriminator: common::MessageProtocol::Soap,
                message_hash: "hash001".to_string(),
                signature_hash: Some("sighash001".to_string()),
                message_body: Some("<soap:Envelope>Msg1</soap:Envelope>".to_string()),
                signature_data: Some(b"<Signature>dummy-sig-1</Signature>".to_vec()),
                response: false,
            })
            .await
            .expect("log message 1");

        let _id2 = db
            .log_message(NewLogRecord {
                query_id: "query-001".to_string(),
                x_request_id: Some("req-1".to_string()),
                client_id_str: Some("DEV/COM/123/CLIENT".to_string()),
                service_id_str: Some("DEV/COM/123/SERVICE".to_string()),
                discriminator: common::MessageProtocol::Soap,
                message_hash: "hash002".to_string(),
                signature_hash: Some("sighash002".to_string()),
                message_body: Some("<soap:Envelope>Msg1Response</soap:Envelope>".to_string()),
                signature_data: Some(b"<Signature>dummy-sig-2</Signature>".to_vec()),
                response: true,
            })
            .await
            .expect("log message 2");

        // Add attachment for record 1
        db.add_attachment(id1, 1, "atthash001", Some(b"Binary attachment data"))
            .await
            .expect("add attachment");

        let temp_dir = std::env::temp_dir().join(format!(
            "xroad-test-archive-{}",
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let archiver = MessageLogArchiver::new(db.clone(), &temp_dir);

        // Run batch archive
        let info = archiver
            .archive_batch()
            .await
            .expect("archive_batch failed")
            .expect("expected ArchiveInfo");

        assert_eq!(info.record_count, 2);
        assert!(info.file_path.exists());
        assert!(!info.last_digest.is_empty());

        // Verify archive
        let result = verify_archive(&info.file_path, None).expect("verify_archive failed");
        assert!(result.valid);
        assert_eq!(result.records_verified, 2);
        assert_eq!(result.last_digest, info.last_digest);

        // Verify DB marks them archived
        let unarchived = db.find_unarchived(10).await.expect("find_unarchived");
        assert!(unarchived.is_empty());

        // Subsequent archive call returns None
        let empty_batch = archiver.archive_batch().await.expect("archive_batch 2");
        assert!(empty_batch.is_none());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
