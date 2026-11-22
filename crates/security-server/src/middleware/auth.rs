use poem::{Endpoint, IntoResponse, Middleware, Request, Response, http::StatusCode};
use serverconf::DatabaseServerConf;
use std::sync::Arc;

/// Authentication middleware for Security Server Admin REST API.
pub struct SecurityServerAuthMiddleware {
    pub server_conf: Arc<DatabaseServerConf>,
}

impl<E: Endpoint> Middleware<E> for SecurityServerAuthMiddleware {
    type Output = SecurityServerAuthEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        SecurityServerAuthEndpoint {
            ep,
            server_conf: self.server_conf.clone(),
        }
    }
}

pub struct SecurityServerAuthEndpoint<E> {
    ep: E,
    server_conf: Arc<DatabaseServerConf>,
}

impl<E: Endpoint> Endpoint for SecurityServerAuthEndpoint<E> {
    type Output = Response;

    async fn call(&self, mut req: Request) -> poem::Result<Self::Output> {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok());

        if let Some(header) = auth_header {
            if let Some(token) = common::headers::parse_api_key_header(header) {
                match self.server_conf.validate_api_key(token).await {
                    Ok(Some(roles)) => {
                        req.extensions_mut()
                            .insert(common::AuthenticatedUser::new(None, roles));
                    }
                    _ => {
                        return Ok(Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .content_type("application/json")
                            .body(r#"{"error":{"code":"invalid_api_key","message":"The API key was not found or is invalid"}}"#));
                    }
                }
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/json")
                    .body(r#"{"error":{"code":"invalid_authorization_header","message":"Invalid Authorization header format"}}"#));
            }
        } else if std::env::var("XROAD_REQUIRE_API_KEY").as_deref() == Ok("true") {
            let path = req.uri().path();
            if !path.starts_with("/swagger-ui")
                && !path.ends_with("/openapi.json")
                && !path.ends_with("/status")
            {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .content_type("application/json")
                    .body(r#"{"error":{"code":"missing_api_key","message":"Missing required Authorization header"}}"#));
            }
        }

        let resp = self.ep.call(req).await?;
        Ok(resp.into_response())
    }
}
