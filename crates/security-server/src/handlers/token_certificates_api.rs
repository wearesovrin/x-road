use crate::handlers::dto::{CertDto, CertificateDetailsDto, ImportCertReq, PossibleActionsDto};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{param::Path, payload::Json, OpenApi};

pub struct TokenCertificatesApi;

#[OpenApi(tag = "ApiTags::TokenCertificates")]
impl TokenCertificatesApi {
    #[oai(path = "/v1/token-certificates", method = "post")]
    pub async fn import_certificate(
        &self,
        _state: Data<&AppState>,
        body: Json<ImportCertReq>,
    ) -> Json<CertDto> {
        let req = body.0;
        Json(CertDto {
            id: req.key_id,
            hash: "cert-hash-1".to_string(),
            active: true,
            status: "SAVED".to_string(),
            certificate_details: Some(CertificateDetailsDto {
                hash: "cert-hash-1".to_string(),
                issuer_distinguished_name: "CN=Security Server CA".to_string(),
                subject_distinguished_name: "CN=Security Server".to_string(),
                not_before: "2020-01-01T00:00:00Z".to_string(),
                not_after: "2030-01-01T00:00:00Z".to_string(),
            }),
            possible_actions: vec!["ACTIVATE".to_string(), "DELETE".to_string()],
        })
    }

    #[oai(path = "/v1/token-certificates/:hash", method = "get")]
    pub async fn get_certificate(
        &self,
        _state: Data<&AppState>,
        hash: Path<String>,
    ) -> Json<CertDto> {
        Json(CertDto {
            id: hash.0.clone(),
            hash: hash.0,
            active: true,
            status: "REGISTERED".to_string(),
            certificate_details: Some(CertificateDetailsDto {
                hash: "hash".to_string(),
                issuer_distinguished_name: "CN=CA".to_string(),
                subject_distinguished_name: "CN=Server".to_string(),
                not_before: "2020-01-01T00:00:00Z".to_string(),
                not_after: "2030-01-01T00:00:00Z".to_string(),
            }),
            possible_actions: vec!["DISABLE".to_string(), "UNREGISTER".to_string()],
        })
    }

    #[oai(path = "/v1/token-certificates/:hash", method = "delete")]
    pub async fn delete_certificate(
        &self,
        _state: Data<&AppState>,
        _hash: Path<String>,
    ) -> Json<bool> {
        Json(true)
    }

    #[oai(path = "/v1/token-certificates/:hash/activate", method = "put")]
    pub async fn activate_certificate(
        &self,
        _state: Data<&AppState>,
        hash: Path<String>,
    ) -> Json<CertDto> {
        Json(CertDto {
            id: hash.0.clone(),
            hash: hash.0,
            active: true,
            status: "REGISTERED".to_string(),
            certificate_details: None,
            possible_actions: vec!["DISABLE".to_string()],
        })
    }

    #[oai(path = "/v1/token-certificates/:hash/disable", method = "put")]
    pub async fn disable_certificate(
        &self,
        _state: Data<&AppState>,
        hash: Path<String>,
    ) -> Json<CertDto> {
        Json(CertDto {
            id: hash.0.clone(),
            hash: hash.0,
            active: false,
            status: "DISABLED".to_string(),
            certificate_details: None,
            possible_actions: vec!["ACTIVATE".to_string()],
        })
    }

    #[oai(path = "/v1/token-certificates/:hash/import", method = "post")]
    pub async fn import_certificate_from_token(
        &self,
        _state: Data<&AppState>,
        hash: Path<String>,
    ) -> Json<CertDto> {
        Json(CertDto {
            id: hash.0.clone(),
            hash: hash.0,
            active: true,
            status: "SAVED".to_string(),
            certificate_details: None,
            possible_actions: vec!["REGISTER".to_string()],
        })
    }

    #[oai(path = "/v1/token-certificates/:hash/possible-actions", method = "get")]
    pub async fn get_possible_actions_for_certificate(
        &self,
        _state: Data<&AppState>,
        _hash: Path<String>,
    ) -> Json<PossibleActionsDto> {
        Json(PossibleActionsDto {
            actions: vec!["ACTIVATE".to_string(), "DELETE".to_string()],
        })
    }

    #[oai(path = "/v1/token-certificates/:hash/register", method = "put")]
    pub async fn register_certificate(
        &self,
        _state: Data<&AppState>,
        hash: Path<String>,
    ) -> Json<CertDto> {
        Json(CertDto {
            id: hash.0.clone(),
            hash: hash.0,
            active: true,
            status: "REGISTRATION_IN_PROGRESS".to_string(),
            certificate_details: None,
            possible_actions: vec!["UNREGISTER".to_string()],
        })
    }

    #[oai(path = "/v1/token-certificates/:hash/unregister", method = "put")]
    pub async fn unregister_certificate(
        &self,
        _state: Data<&AppState>,
        hash: Path<String>,
    ) -> Json<CertDto> {
        Json(CertDto {
            id: hash.0.clone(),
            hash: hash.0,
            active: false,
            status: "SAVED".to_string(),
            certificate_details: None,
            possible_actions: vec!["REGISTER".to_string()],
        })
    }

    #[oai(
        path = "/v1/token-certificates/:hash/mark-for-deletion",
        method = "put"
    )]
    pub async fn mark_certificate_for_deletion(
        &self,
        _state: Data<&AppState>,
        hash: Path<String>,
    ) -> Json<CertDto> {
        Json(CertDto {
            id: hash.0.clone(),
            hash: hash.0,
            active: false,
            status: "DELETION_IN_PROGRESS".to_string(),
            certificate_details: None,
            possible_actions: vec!["DELETE".to_string()],
        })
    }
}
