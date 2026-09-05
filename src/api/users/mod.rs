mod auth;
mod me;

use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi()]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(auth::openapi());
    openapi.merge(me::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(auth::configure).configure(me::configure);
}

/// Authenticated username; preserves the existing empty HTTP 400 rejection.
pub(crate) struct SessionUser(pub String);
impl actix_web::FromRequest for SessionUser {
    type Error = actix_web::Error;
    type Future = std::future::Ready<Result<Self, Self::Error>>;
    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        use actix_session::SessionExt;
        std::future::ready(match req.get_session().get::<String>("username") {
            Ok(Some(username)) => Ok(Self(username)),
            _ => Err(actix_web::error::InternalError::from_response(
                "missing session user",
                actix_web::HttpResponse::BadRequest().finish(),
            )
            .into()),
        })
    }
}

/// HSR file import authorization, kept at the endpoint boundary.
pub(crate) async fn verified_or_admin(
    username: &str,
    uid: i32,
    pool: &sqlx::PgPool,
) -> anyhow::Result<(bool, bool)> {
    let admin = crate::database::admins::exists(username, pool).await?;
    let allowed = admin
        || crate::database::connections::get_by_username(username, pool)
            .await?
            .iter()
            .any(|connection| connection.uid == uid && connection.verified);
    Ok((admin, allowed))
}

#[cfg(test)]
mod session_user_tests {
    use super::*;
    use actix_session::SessionExt;
    use actix_web::{test::TestRequest, FromRequest};
    #[actix_web::test]
    async fn missing_and_malformed_user_reject_with_empty_400() {
        for stored in [None, Some(serde_json::json!(123))] {
            let request = TestRequest::default().to_http_request();
            request.get_session().insert("other", "value").unwrap();
            if let Some(value) = stored {
                request.get_session().insert("username", value).unwrap();
            }
            let result =
                SessionUser::from_request(&request, &mut actix_web::dev::Payload::None).await;
            let error = match result {
                Err(error) => error,
                Ok(_) => panic!("must reject"),
            };
            let response = error.error_response();
            assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
            assert!(actix_web::body::to_bytes(response.into_body())
                .await
                .unwrap()
                .is_empty());
        }
        let request = TestRequest::default().to_http_request();
        request.get_session().insert("username", "alice").unwrap();
        let user = SessionUser::from_request(&request, &mut actix_web::dev::Payload::None)
            .await
            .unwrap();
        assert_eq!(user.0, "alice");
    }
}
