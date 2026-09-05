use std::collections::HashMap;

use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/auth/login")),
    paths(login),
    components(schemas(UserLogin))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum UserLogin {
    UsernamePassword { username: String, password: String },
    Token { token: String },
}

#[utoipa::path(
    tag = "users/auth/login",
    post,
    path = "/api/users/auth/login",
    request_body(content = UserLogin,
        examples(
            ("UsernamePassword" = (value = json!({"username": "meow", "password": "meow12"}))),
            ("Token" = (value = json!({"token": "a3449001-8762-48e2-8813-3abe92a29884"})))
        )
    ),
    responses(
        (status = 200, description = "Successfull login. The session id is returned in a cookie named `id`. You need to include this cookie in subsequent requests"),
        (status = 400, description = "Don't have an account")
    )
)]
#[post("/api/users/auth/login")]
async fn login(
    session: Session,
    user_login: web::Json<UserLogin>,
    tokens: web::Data<Mutex<HashMap<Uuid, String>>>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let username = match &*user_login {
        UserLogin::UsernamePassword { username, password } => {
            let username = username.trim().to_lowercase();

            let Ok(user) = database::users::get_one_by_username(&username, &pool).await else {
                return Ok(HttpResponse::BadRequest().finish());
            };

            if !argon2::verify_encoded(&user.password, password.as_bytes()).unwrap_or_default() {
                return Ok(HttpResponse::BadRequest().finish());
            }

            username.clone()
        }
        UserLogin::Token { token } => {
            let Ok(token) = token.parse::<Uuid>() else {
                return Ok(HttpResponse::BadRequest().finish());
            };
            let Some(username) = tokens.lock().await.remove(&token) else {
                return Ok(HttpResponse::BadRequest().finish());
            };

            username.clone()
        }
    };

    session.insert("username", username.to_lowercase())?;

    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn malformed_token_is_an_empty_400_without_database_access() {
        // The malformed-token path must reject before touching PostgreSQL or
        // consuming any valid one-time token in the shared store.
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://unused:unused@localhost/unused")
            .unwrap();
        let valid_token = Uuid::new_v4();
        let tokens = web::Data::new(Mutex::new(HashMap::from([(
            valid_token,
            "alice".to_string(),
        )])));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool))
                .app_data(tokens.clone())
                .service(login),
        )
        .await;
        let request = test::TestRequest::post()
            .uri("/api/users/auth/login")
            .set_json(serde_json::json!({"token":"not-a-uuid"}))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::BAD_REQUEST);
        assert!(test::read_body(response).await.is_empty());
        assert!(tokens.lock().await.contains_key(&valid_token));
    }
}
