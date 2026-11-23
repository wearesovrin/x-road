use crate::handlers::ApiTags;
use crate::handlers::dto::BackupDto;
use poem_openapi::{
    OpenApi,
    param::Path,
    payload::{Attachment, Json},
};

pub struct BackupsApi;

#[OpenApi(tag = "ApiTags::Backups")]
impl BackupsApi {
    #[oai(path = "/v1/backups", method = "get")]
    pub async fn list_backups(&self) -> Json<Vec<BackupDto>> {
        Json(vec![BackupDto {
            filename: "backup_latest.tar.gz".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: 4096,
        }])
    }

    #[oai(path = "/v1/backups", method = "post")]
    pub async fn create_backup(&self) -> Json<BackupDto> {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        Json(BackupDto {
            filename: format!("conf_backup_{ts}.tar.gz"),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: 4096,
        })
    }

    #[oai(path = "/v1/backups/:filename/download", method = "get")]
    pub async fn download_backup(&self, filename: Path<String>) -> Attachment<Vec<u8>> {
        Attachment::new(vec![0x1f, 0x8b, 0x08, 0x00]).filename(filename.0)
    }

    #[oai(path = "/v1/backups/:filename/restore", method = "post")]
    pub async fn restore_backup(&self, _filename: Path<String>) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/backups/:filename", method = "delete")]
    pub async fn delete_backup(&self, _filename: Path<String>) -> Json<bool> {
        Json(true)
    }
}
