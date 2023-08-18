use actix_session::Session;
use actix_web::{put, web, HttpResponse, Responder};
use argon2::Config;
use rand::Rng;
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::api::ApiResult;

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/password")),
    paths(put_password),
    components(schemas(
        PasswordUpdate
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_password);
}

#[derive(Deserialize, ToSchema)]
pub struct PasswordUpdate {
    password: String,
}

#[utoipa::path(
    tag = "users/me/password",
    put,
    path = "/api/users/me/password",
    request_body = PasswordUpdate,
    responses(
        (status = 200, description = "Updated password"),
    )
)]
#[put("/api/users/me/password")]
async fn put_password(
    session: Session,
    password_update: web::Json<PasswordUpdate>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let salt = rand::thread_rng().gen::<[u8; 32]>();

    let password = argon2::hash_encoded(
        password_update.password.as_bytes(),
        &salt,
        &Config::rfc9106_low_mem(),
    )?;

    stardb_database::update_user_password(&username, &password, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
