use actix_session::Session;
use actix_web::{put, web, HttpResponse, Responder};
use rand::{distributions::Alphanumeric, Rng};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    database::{self, DbVerification},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/verifications/{uid}")),
    paths(put_verification)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_verification);
}

#[utoipa::path(
    tag = "users/me/verifications/{uid}",
    put,
    path = "/api/users/me/verifications/{uid}",
    responses(
        (status = 200, description = "Added verification", body = String),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/verifications/{uid}")]
async fn put_verification(
    session: Session,
    uid: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let db_verification = DbVerification {
        uid: *uid,
        username,
        token,
    };

    database::set_verification(&db_verification, &pool).await?;

    Ok(HttpResponse::Ok().json(db_verification.token))
}
