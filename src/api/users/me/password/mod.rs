use crate::api::users::SessionUser;
use actix_web::{put, web, HttpResponse, Responder};
use argon2::Config;
use rand::RngExt;
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/password")),
    paths(put_password),
    components(schemas(
        PasswordUpdate
    ))
)]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module's routes and any shared application data.
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
/// Replaces the authenticated user's password with a newly salted Argon2 hash.
async fn put_password(
    SessionUser(username): SessionUser,
    password_update: web::Json<PasswordUpdate>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let salt = rand::rng().random::<[u8; 32]>();

    let password = argon2::hash_encoded(
        password_update.password.as_bytes(),
        &salt,
        &Config::rfc9106_low_mem(),
    )?;

    database::users::update_password_by_username(&username, &password, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
