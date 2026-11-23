use crate::handlers::{ApiTags, AppState};
use crate::handlers::dto::ConfigurationAnchorDto;
use poem::{http::StatusCode, web::Data};
use poem_openapi::{
    OpenApi,
    param::Path,
    payload::{Attachment, Json},
};

pub struct ConfigurationSourcesApi;

#[OpenApi(tag = "ApiTags::ConfigurationSources")]
impl ConfigurationSourcesApi {
    #[oai(
        path = "/v1/configuration-sources/:configuration_type/anchor",
        method = "get"
    )]
    pub async fn get_configuration_anchor(
        &self,
        state: Data<&AppState>,
        _configuration_type: Path<String>,
    ) -> poem::Result<Json<ConfigurationAnchorDto>> {
        Ok(Json(ConfigurationAnchorDto {
            instance_identifier: state.instance_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }))
    }

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/anchor/download",
        method = "get"
    )]
    pub async fn download_configuration_anchor(
        &self,
        state: Data<&AppState>,
        _configuration_type: Path<String>,
    ) -> poem::Result<Attachment<Vec<u8>>> {
        let xml = state
            .db
            .generate_anchor_xml(&state.instance_id)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        Ok(Attachment::new(xml.into_bytes())
            .filename(format!("configuration-anchor-{}.xml", state.instance_id)))
    }
}
