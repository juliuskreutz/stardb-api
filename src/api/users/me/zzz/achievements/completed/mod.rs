mod id;

use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/zzz/achievements/completed")),
    paths(get_zzz_user_achievements_completed, put_zzz_user_achievements_completed)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_zzz_user_achievements_completed)
        .service(put_zzz_user_achievements_completed)
        .configure(id::configure);
}

#[utoipa::path(
    tag = "users/me/zzz/achievements/completed",
    get,
    path = "/api/users/me/zzz/achievements/completed",
    responses(
        (status = 200, description = "Achievement ids", body = Vec<i64>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/zzz/achievements/completed")]
async fn get_zzz_user_achievements_completed(
    session: Session,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let completed: Vec<_> =
        database::zzz::users_achievements_completed::get_by_username(&username, &pool)
            .await?
            .iter()
            .map(|c| c.id)
            .collect();

    Ok(HttpResponse::Ok().json(completed))
}

#[utoipa::path(
    tag = "users/me/zzz/achievements/completed",
    put,
    path = "/api/users/me/zzz/achievements/completed",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/zzz/achievements/completed")]
async fn put_zzz_user_achievements_completed(
    session: Session,
    ids: web::Json<Vec<i32>>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let mut complete =
        database::zzz::users_achievements_completed::DbUserAchievementCompleted { username, id: 0 };

    for id in ids.0 {
        complete.id = id;

        database::zzz::users_achievements_completed::add(&complete, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
