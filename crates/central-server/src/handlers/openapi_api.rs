use crate::handlers::ApiTags;
use poem_openapi::{payload::PlainText, OpenApi};

pub struct OpenapiApi;

#[OpenApi(tag = "ApiTags::Openapi")]
impl OpenapiApi {
    #[oai(path = "/v1/openapi.yaml", method = "get")]
    pub async fn download_openapi(&self) -> PlainText<&'static str> {
        PlainText(include_str!(
            "../../../../java/src/central-server/openapi-model/src/main/resources/openapi-definition.yaml"
        ))
    }
}
