use crate::handlers::dto::ConfigurationAnchorDto;
use crate::services::CentralServerDb;
use poem::http::StatusCode;
use poem_openapi::{
    OpenApi,
    param::Path,
    payload::{Attachment, Json},
};

pub struct ConfigurationSourcesApi {
    pub instance_id: String,
    pub db: CentralServerDb,
}

#[OpenApi]
impl ConfigurationSourcesApi {
    #[oai(
        path = "/v1/configuration-sources/:configuration_type/anchor",
        method = "get"
    )]
    pub async fn get_configuration_anchor(
        &self,
        _configuration_type: Path<String>,
    ) -> poem::Result<Json<ConfigurationAnchorDto>> {
        Ok(Json(ConfigurationAnchorDto {
            instance_identifier: self.instance_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }))
    }

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/anchor/download",
        method = "get"
    )]
    pub async fn download_configuration_anchor(
        &self,
        _configuration_type: Path<String>,
    ) -> poem::Result<Attachment<Vec<u8>>> {
        let xml = self
            .db
            .generate_anchor_xml(&self.instance_id)
            .await
            .map_err(|e| {
                poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
            })?;

        Ok(Attachment::new(xml.into_bytes())
            .filename(format!("configuration-anchor-{}.xml", self.instance_id)))
    }

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/anchor/re-create",
        method = "post"
    )]
    pub async fn recreate_configuration_anchor(
        &self,
        _configuration_type: Path<String>,
    ) -> poem::Result<Json<ConfigurationAnchorDto>> {
        Ok(Json(ConfigurationAnchorDto {
            instance_identifier: self.instance_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
        }))
    }
}
