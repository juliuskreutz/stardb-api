mod uid;

use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::{self, DbVerification},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/verifications")),
    paths(get_verifications),
    components(schemas(
        Verification
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(uid::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_verifications).configure(uid::configure);
}

#[derive(Serialize, ToSchema)]
pub struct Verification {
    uid: i64,
    otp: String,
}

impl From<DbVerification> for Verification {
    fn from(db_verification: DbVerification) -> Self {
        Verification {
            uid: db_verification.uid,
            otp: db_verification.token,
        }
    }
}

#[utoipa::path(
    tag = "users/me/verifications",
    get,
    path = "/api/users/me/verifications",
    responses(
        (status = 200, description = "Verifications", body = Vec<Verification>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/verifications")]
async fn get_verifications(session: Session, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let db_verifications = database::get_verifications_by_username(&username, &pool).await?;

    let verifications: Vec<_> = db_verifications
        .into_iter()
        .map(Verification::from)
        .collect();

    Ok(HttpResponse::Ok().json(verifications))
}
