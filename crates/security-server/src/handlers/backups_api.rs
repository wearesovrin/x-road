use crate::handlers::dto::{BackupDto, BackupExtDto, BackupUploadReq};
use crate::handlers::ApiTags;
use poem_openapi::{
    param::Path,
    payload::{Attachment, Json},
    OpenApi,
};
use std::sync::{LazyLock, RwLock};

static BACKUP_STORE: LazyLock<RwLock<Vec<BackupDto>>> = LazyLock::new(|| RwLock::new(Vec::new()));

pub struct BackupsApi;

#[OpenApi(tag = "ApiTags::Backups")]
impl BackupsApi {
    #[oai(path = "/v1/backups", method = "get")]
    pub async fn list_backups(&self) -> Json<Vec<BackupDto>> {
        let store = BACKUP_STORE.read().unwrap();
        Json(store.clone())
    }

    #[oai(path = "/v1/backups", method = "post")]
    pub async fn create_backup(&self) -> Json<BackupDto> {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let dto = BackupDto {
            filename: format!("ss_backup_{ts}.tar.gz"),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: 4096,
        };
        BACKUP_STORE.write().unwrap().push(dto.clone());
        Json(dto)
    }

    #[oai(path = "/v1/backups/ext", method = "post")]
    pub async fn create_backup_ext(&self) -> Json<BackupExtDto> {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("ss_backup_ext_{ts}.tar.gz");
        let dto = BackupDto {
            filename: filename.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: 4096,
        };
        BACKUP_STORE.write().unwrap().push(dto);
        Json(BackupExtDto {
            filename,
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: 4096,
            is_archive: true,
        })
    }

    #[oai(path = "/v1/backups/upload", method = "post")]
    pub async fn upload_backup(&self, body: Json<BackupUploadReq>) -> Json<BackupDto> {
        let dto = BackupDto {
            filename: body.0.filename,
            created_at: chrono::Utc::now().to_rfc3339(),
            size_bytes: body.0.content_base64.len(),
        };
        BACKUP_STORE.write().unwrap().push(dto.clone());
        Json(dto)
    }

    #[oai(path = "/v1/backups/:filename", method = "delete")]
    pub async fn delete_backup(&self, filename: Path<String>) -> Json<bool> {
        BACKUP_STORE
            .write()
            .unwrap()
            .retain(|b| b.filename != filename.0);
        Json(true)
    }

    #[oai(path = "/v1/backups/:filename/download", method = "get")]
    pub async fn download_backup(&self, filename: Path<String>) -> Attachment<Vec<u8>> {
        Attachment::new(vec![0x1f, 0x8b, 0x08, 0x00]).filename(filename.0)
    }

    #[oai(path = "/v1/backups/:filename/restore", method = "put")]
    pub async fn restore_backup(&self, _filename: Path<String>) -> Json<bool> {
        Json(true)
    }
}
