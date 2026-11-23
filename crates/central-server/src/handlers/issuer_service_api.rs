use crate::handlers::dto::{IssuerServiceHostRequest, IssuerServiceStateDto};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{payload::Json, OpenApi};

pub struct IssuerServiceApi;

#[OpenApi(tag = "ApiTags::IssuerService")]
impl IssuerServiceApi {
    #[oai(path = "/v1/issuer-service", method = "get")]
    pub async fn get_issuer_service_state(
        &self,
        _state: Data<&AppState>,
    ) -> Json<IssuerServiceStateDto> {
        Json(IssuerServiceStateDto {
            host: Some("issuer.example.org".to_string()),
            bound: true,
            host_set: true,
            certificate_ready: true,
            run_in_progress: false,
            did_to_bind: None,
            bound_did: Some("did:xroad:issuer1".to_string()),
            can_sign: true,
            status: "READY".to_string(),
        })
    }

    #[oai(path = "/v1/issuer-service/host", method = "put")]
    pub async fn set_issuer_service_host(
        &self,
        _state: Data<&AppState>,
        body: Json<IssuerServiceHostRequest>,
    ) -> Json<IssuerServiceStateDto> {
        Json(IssuerServiceStateDto {
            host: Some(body.0.host),
            bound: false,
            host_set: true,
            certificate_ready: true,
            run_in_progress: false,
            did_to_bind: Some("did:xroad:issuer1".to_string()),
            bound_did: None,
            can_sign: true,
            status: "INITIALIZING".to_string(),
        })
    }

    #[oai(path = "/v1/issuer-service/initialize", method = "post")]
    pub async fn initialize_issuer_service(
        &self,
        _state: Data<&AppState>,
    ) -> Json<IssuerServiceStateDto> {
        Json(IssuerServiceStateDto {
            host: Some("issuer.example.org".to_string()),
            bound: true,
            host_set: true,
            certificate_ready: true,
            run_in_progress: false,
            did_to_bind: None,
            bound_did: Some("did:xroad:issuer1".to_string()),
            can_sign: true,
            status: "READY".to_string(),
        })
    }
}
