use poem_openapi::{OpenApi, payload::Json};

use super::AdminApiState;
use super::dto::{ArchiveResponse, LogRecordResponse};

pub struct MessageLogApi {
    pub state: AdminApiState,
}

#[OpenApi]
impl MessageLogApi {
    /// Retrieves recent unarchived audit log records from the message log.
    #[oai(path = "/v1/messagelog/recent", method = "get")]
    pub async fn recent_logs(&self) -> Json<Vec<LogRecordResponse>> {
        let records = self
            .state
            .message_log
            .find_unarchived(50)
            .await
            .unwrap_or_default();
        let list = records
            .into_iter()
            .map(|r| LogRecordResponse {
                id: r.id,
                query_id: r.query_id,
                client_id: r.client_id_str,
                service_id: r.service_id_str,
                archived: r.archived,
                timestamp: r.time.to_rfc3339(),
            })
            .collect();
        Json(list)
    }

    /// Triggers log archival of unarchived messages to gzip archives.
    #[oai(path = "/v1/messagelog/archive", method = "post")]
    pub async fn archive_logs(&self) -> Json<ArchiveResponse> {
        let archive_dir =
            std::env::var("XROAD_ARCHIVE_DIR").unwrap_or_else(|_| "/tmp/xroad/archive".to_string());
        let archiver =
            messagelog::MessageLogArchiver::new((*self.state.message_log).clone(), archive_dir)
                .with_max_records(100);

        match archiver.archive_batch().await {
            Ok(Some(info)) => Json(ArchiveResponse {
                success: true,
                records_archived: info.record_count,
                file_path: Some(info.file_path.to_string_lossy().to_string()),
                file_name: Some(info.file_name),
                last_digest: Some(info.last_digest),
                message: format!("Successfully archived {} records", info.record_count),
            }),
            Ok(None) => Json(ArchiveResponse {
                success: true,
                records_archived: 0,
                file_path: None,
                file_name: None,
                last_digest: None,
                message: "No unarchived records to process".to_string(),
            }),
            Err(e) => Json(ArchiveResponse {
                success: false,
                records_archived: 0,
                file_path: None,
                file_name: None,
                last_digest: None,
                message: format!("Archival failed: {e}"),
            }),
        }
    }
}
