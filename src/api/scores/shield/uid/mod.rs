use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{scores::shield::ScoreShield, ApiResult},
    database::{self, DbScoreShield},
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores/shield/{uid}")),
    paths(get_score_shield, put_score_shield),
    components(schemas(
        ShieldUpdate
    ))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
pub struct ShieldUpdate {
    pub shield: i32,
    pub video: String,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_score_shield).service(put_score_shield);
}

#[utoipa::path(
    tag = "scores/shield/{uid}",
    get,
    path = "/api/scores/shield/{uid}",
    responses(
        (status = 200, description = "[ScoreShield]", body = Vec<ScoreShield>),
    )
)]
#[get("/api/scores/shield/{uid}")]
async fn get_score_shield(
    uid: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let score: ScoreShield = database::get_score_shield_by_uid(*uid, &pool).await?.into();

    Ok(HttpResponse::Ok().json(score))
}

#[utoipa::path(
    tag = "scores/shield/{uid}",
    put,
    path = "/api/scores/shield/{uid}",
    request_body = ShieldUpdate,
    responses(
        (status = 200, description = "ScoreShield updated"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/scores/shield/{uid}")]
async fn put_score_shield(
    session: Session,
    uid: web::Path<i64>,
    shield_update: web::Json<ShieldUpdate>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::get_admin_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let uid = *uid;
    let shield = shield_update.shield;
    let video = shield_update.video.clone();

    let db_set_score_shield = DbScoreShield {
        uid,
        shield,
        video,
        ..Default::default()
    };

    database::set_score_shield(&db_set_score_shield, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
