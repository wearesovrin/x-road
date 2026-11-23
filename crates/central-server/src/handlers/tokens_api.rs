use crate::handlers::dto::{TokenDto, TokenPasswordRequest};
use crate::handlers::{ApiTags, AppState};
use poem::web::Data;
use poem_openapi::{param::Path, payload::Json, OpenApi};

pub struct TokensApi;

#[OpenApi(tag = "ApiTags::Tokens")]
impl TokensApi {
    #[oai(path = "/v1/tokens", method = "get")]
    pub async fn get_tokens(&self, _state: Data<&AppState>) -> Json<Vec<TokenDto>> {
        Json(vec![TokenDto {
            id: "softToken-0".to_string(),
            name: "SoftToken 0".to_string(),
            serial_number: Some("0000".to_string()),
            status: "OK".to_string(),
            logged_in: true,
            active: true,
            available: true,
            possible_actions: vec!["LOGOUT".to_string()],
            keys: vec![],
        }])
    }

    #[oai(path = "/v1/tokens/:token_id/login", method = "put")]
    pub async fn login_token(
        &self,
        _state: Data<&AppState>,
        token_id: Path<String>,
        _body: Json<TokenPasswordRequest>,
    ) -> Json<TokenDto> {
        Json(TokenDto {
            id: token_id.0,
            name: "SoftToken 0".to_string(),
            serial_number: Some("0000".to_string()),
            status: "OK".to_string(),
            logged_in: true,
            active: true,
            available: true,
            possible_actions: vec!["LOGOUT".to_string()],
            keys: vec![],
        })
    }

    #[oai(path = "/v1/tokens/:token_id/logout", method = "put")]
    pub async fn logout_token(
        &self,
        _state: Data<&AppState>,
        token_id: Path<String>,
    ) -> Json<TokenDto> {
        Json(TokenDto {
            id: token_id.0,
            name: "SoftToken 0".to_string(),
            serial_number: Some("0000".to_string()),
            status: "OK".to_string(),
            logged_in: false,
            active: true,
            available: true,
            possible_actions: vec!["LOGIN".to_string()],
            keys: vec![],
        })
    }
}
