use std::collections::HashMap;

use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{database, Result};

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
) -> Result<impl Responder> {
    let username = match &*user_login {
        UserLogin::UsernamePassword { username, password } => {
            let Ok(user) = database::get_user_by_username(username, &pool).await else {
                return Ok(HttpResponse::BadRequest().finish());
            };

            if !argon2::verify_encoded(&user.password, password.as_bytes()).unwrap_or_default() {
                return Ok(HttpResponse::BadRequest().finish());
            }

            username.clone()
        }
        UserLogin::Token { token } => {
            let Some(username) = tokens.lock().await.remove(&token.parse()?) else {
                return Ok(HttpResponse::BadRequest().finish());
            };

            username.clone()
        }
    };

    let user = database::get_user_by_username(&username, &pool).await?;

    session.insert("username", user.username)?;
    session.insert("admin", user.admin)?;

    Ok(HttpResponse::Ok().finish())
}
