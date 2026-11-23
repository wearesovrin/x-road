use crate::handlers::dto::{
    AddSigningKeyRequest, ConfigurationAnchorDto, ConfigurationPartDto, SigningKeyDto,
    UploadConfigurationPartRequest,
};
use crate::handlers::{ApiTags, AppState};
use poem::{http::StatusCode, web::Data};
use poem_openapi::{
    param::Path,
    payload::{Attachment, Json, PlainText},
    OpenApi,
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
            hash: "anchor-hash-example".to_string(),
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

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/anchor/re-create",
        method = "put"
    )]
    pub async fn recreate_configuration_anchor(
        &self,
        state: Data<&AppState>,
        _configuration_type: Path<String>,
    ) -> poem::Result<Json<ConfigurationAnchorDto>> {
        Ok(Json(ConfigurationAnchorDto {
            instance_identifier: state.instance_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            hash: "recreated-anchor-hash".to_string(),
        }))
    }

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/configuration-parts",
        method = "get"
    )]
    pub async fn get_configuration_parts(
        &self,
        _state: Data<&AppState>,
        _configuration_type: Path<String>,
    ) -> Json<Vec<ConfigurationPartDto>> {
        Json(vec![
            ConfigurationPartDto {
                content_identifier: "SHARED-PARAMETERS".to_string(),
                file_name: "shared-params.xml".to_string(),
                version: 2,
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
            ConfigurationPartDto {
                content_identifier: "MONITORING-PARAMETERS".to_string(),
                file_name: "monitoring-params.xml".to_string(),
                version: 1,
                updated_at: chrono::Utc::now().to_rfc3339(),
            },
        ])
    }

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/configuration-parts",
        method = "post"
    )]
    pub async fn upload_configuration_parts(
        &self,
        _state: Data<&AppState>,
        _configuration_type: Path<String>,
        body: Json<UploadConfigurationPartRequest>,
    ) -> Json<ConfigurationPartDto> {
        Json(ConfigurationPartDto {
            content_identifier: body.0.content_identifier,
            file_name: "custom-part.xml".to_string(),
            version: 1,
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/configuration-parts/:content_identifier/:version/download",
        method = "get"
    )]
    pub async fn download_configuration_parts(
        &self,
        state: Data<&AppState>,
        _configuration_type: Path<String>,
        content_identifier: Path<String>,
        _version: Path<i32>,
    ) -> poem::Result<Attachment<Vec<u8>>> {
        let xml = state
            .db
            .generate_shared_params_xml(&state.instance_id)
            .await
            .unwrap_or_else(|_| "<shared-params/>".to_string());
        Ok(
            Attachment::new(xml.into_bytes()).filename(format!("{}.xml", content_identifier.0))
        )
    }

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/signing-keys",
        method = "post"
    )]
    pub async fn add_signing_key(
        &self,
        _state: Data<&AppState>,
        _configuration_type: Path<String>,
        body: Json<AddSigningKeyRequest>,
    ) -> Json<SigningKeyDto> {
        Json(SigningKeyDto {
            id: "key-1".to_string(),
            key_hash: "key-hash-1".to_string(),
            label: body.0.label.unwrap_or_else(|| "signing-key".to_string()),
            active: true,
            available: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    #[oai(
        path = "/v1/configuration-sources/:configuration_type/download-url",
        method = "get"
    )]
    pub async fn get_download_url(
        &self,
        _state: Data<&AppState>,
        configuration_type: Path<String>,
    ) -> PlainText<String> {
        PlainText(format!(
            "http://127.0.0.1:4000/internalconf/{}",
            configuration_type.0
        ))
    }

    #[oai(path = "/v1/signing-keys/:sign_key_id", method = "delete")]
    pub async fn delete_signing_key(
        &self,
        _state: Data<&AppState>,
        _sign_key_id: Path<String>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/signing-keys/:sign_key_id/activate", method = "put")]
    pub async fn activate_signing_key(
        &self,
        _state: Data<&AppState>,
        sign_key_id: Path<String>,
    ) -> Json<SigningKeyDto> {
        Json(SigningKeyDto {
            id: sign_key_id.0,
            key_hash: "activated-hash".to_string(),
            label: "active-key".to_string(),
            active: true,
            available: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}
