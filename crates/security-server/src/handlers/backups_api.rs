use chrono::Utc;
use poem::web::Data;
use poem_openapi::{OpenApi, param::Path, payload::Json};

use super::{ApiTags, AppState};
use super::dto::{BackupDto, SimpleSuccessResponse};

pub struct BackupsApi;

#[OpenApi(tag = "ApiTags::Backups")]
impl BackupsApi {
    /// Lists configuration backups.
    #[oai(path = "/v1/backups", method = "get")]
    pub async fn list_backups(&self, state: Data<&AppState>) -> Json<Vec<BackupDto>> {
        let settings = state.system_settings.read().await;
        Json(settings.backups.clone())
    }

    /// Creates a configuration backup.
    #[oai(path = "/v1/backups", method = "post")]
    pub async fn create_backup(&self, state: Data<&AppState>) -> Json<BackupDto> {
        let filename = format!("backup-{}.tar.gz", Utc::now().format("%Y%m%d_%H%M%S"));
        let backup = BackupDto {
            filename: filename.clone(),
            created_at: Utc::now().to_rfc3339(),
            size_bytes: 4096,
        };

        let mut settings = state.system_settings.write().await;
        settings.backups.push(backup.clone());
        Json(backup)
    }

    /// Deletes a backup by filename.
    #[oai(path = "/v1/backups/:filename", method = "delete")]
    pub async fn delete_backup(
        &self,
        state: Data<&AppState>,
        filename: Path<String>,
    ) -> Json<SimpleSuccessResponse> {
        let mut settings = state.system_settings.write().await;
        settings.backups.retain(|b| b.filename != filename.0);
        Json(SimpleSuccessResponse {
            success: true,
            message: format!("Backup {} deleted", filename.0),
        })
    }
}
