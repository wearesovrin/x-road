use crate::services::CentralServerDb;
use poem::{Endpoint, IntoResponse, Middleware, Request, Response, http::StatusCode};

/// Authentication middleware for Central Server Admin REST API.
pub struct CentralServerAuthMiddleware {
    pub db: CentralServerDb,
}

impl<E: Endpoint> Middleware<E> for CentralServerAuthMiddleware {
    type Output = CentralServerAuthEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        CentralServerAuthEndpoint {
            ep,
            db: self.db.clone(),
        }
    }
}

pub struct CentralServerAuthEndpoint<E> {
    ep: E,
    db: CentralServerDb,
}

impl<E: Endpoint> Endpoint for CentralServerAuthEndpoint<E> {
    type Output = Response;

    async fn call(&self, mut req: Request) -> poem::Result<Self::Output> {
        let auth_header = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok());

        if let Some(header) = auth_header {
            if let Some(token) = common::headers::parse_api_key_header(header) {
                match self.db.validate_api_key(token).await {
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
                && !path.contains("shared-params.xml")
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
