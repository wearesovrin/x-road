use crate::handlers::dto::{InitialRegistrationDto, InitializationStatusDto};
use crate::services::CentralServerDb;
use poem::http::StatusCode;
use poem_openapi::{OpenApi, payload::Json};

pub struct InitializationApi {
    pub db: CentralServerDb,
}

#[OpenApi]
impl InitializationApi {
    #[oai(path = "/v1/initialization/status", method = "get")]
    pub async fn get_initialization_status(&self) -> Json<InitializationStatusDto> {
        let is_init = self
            .db
            .get_system_parameter("is_initialized")
            .await
            .ok()
            .flatten()
            .map(|v| v == "true")
            .unwrap_or(false);
        Json(InitializationStatusDto {
            is_initialized: is_init,
        })
    }

    #[oai(path = "/v1/initialization", method = "post")]
    pub async fn initialize(
        &self,
        body: Json<InitialRegistrationDto>,
    ) -> poem::Result<Json<InitializationStatusDto>> {
        self.db
            .set_server_address(&body.0.central_server_address)
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;
        self.db
            .set_system_parameter("is_initialized", "true")
            .await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(InitializationStatusDto {
            is_initialized: true,
        }))
    }
}
