use crate::handlers::dto::{TrustedAnchorDto, UploadTrustedAnchorRequest};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{
    param::Path,
    payload::{Attachment, Json},
    OpenApi,
};

pub struct TrustedAnchorsApi;

#[OpenApi(tag = "ApiTags::TrustedAnchors")]
impl TrustedAnchorsApi {
    #[oai(path = "/v1/trusted-anchors", method = "get")]
    pub async fn get_trusted_anchors(&self, state: Data<&AppState>) -> Json<Vec<TrustedAnchorDto>> {
        Json(vec![TrustedAnchorDto {
            hash: "anchor-hash-01".to_string(),
            instance_identifier: state.instance_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }])
    }

    #[oai(path = "/v1/trusted-anchors", method = "post")]
    pub async fn upload_trusted_anchor(
        &self,
        state: Data<&AppState>,
        _body: Json<UploadTrustedAnchorRequest>,
    ) -> Json<TrustedAnchorDto> {
        Json(TrustedAnchorDto {
            hash: "uploaded-anchor-hash".to_string(),
            instance_identifier: state.instance_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    #[oai(path = "/v1/trusted-anchors/preview", method = "post")]
    pub async fn preview_trusted_anchor(
        &self,
        state: Data<&AppState>,
        _body: Json<UploadTrustedAnchorRequest>,
    ) -> Json<TrustedAnchorDto> {
        Json(TrustedAnchorDto {
            hash: "preview-anchor-hash".to_string(),
            instance_identifier: state.instance_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    #[oai(path = "/v1/trusted-anchors/:hash", method = "delete")]
    pub async fn delete_trusted_anchor(
        &self,
        _state: Data<&AppState>,
        _hash: Path<String>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/trusted-anchors/:hash/download", method = "get")]
    pub async fn download_trusted_anchor(
        &self,
        state: Data<&AppState>,
        hash: Path<String>,
    ) -> Attachment<Vec<u8>> {
        let xml = format!(
            "<trusted-anchor><instance>{}</instance><hash>{}</hash></trusted-anchor>",
            state.instance_id, hash.0
        );
        Attachment::new(xml.into_bytes()).filename(format!("{}.xml", hash.0))
    }
}
