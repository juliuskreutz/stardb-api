mod id;

use actix_session::Session;
use actix_web::{delete, get, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/achievements/completed")),
    paths(get_user_achievements_completed, put_user_achievements_completed, delete_user_achievements_completed)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_achievements_completed)
        .service(put_user_achievements_completed)
        .service(delete_user_achievements_completed)
        .configure(id::configure);
}

#[utoipa::path(
    tag = "users/me/achievements/completed",
    get,
    path = "/api/users/me/achievements/completed",
    responses(
        (status = 200, description = "Achievement ids", body = Vec<i64>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/achievements/completed")]
async fn get_user_achievements_completed(
    session: Session,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let completed: Vec<_> = database::get_user_achievements_completed_by_username(&username, &pool)
        .await?
        .iter()
        .map(|c| c.id)
        .collect();

    Ok(HttpResponse::Ok().json(completed))
}

#[utoipa::path(
    tag = "users/me/achievements/completed",
    put,
    path = "/api/users/me/achievements/completed",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/achievements/completed")]
async fn put_user_achievements_completed(
    session: Session,
    ids: web::Json<Vec<i32>>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let mut complete = database::DbUserAchievementCompleted { username, id: 0 };

    for id in ids.0 {
        complete.id = id;

        database::add_user_achievement_completed(&complete, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/achievements/completed",
    delete,
    path = "/api/users/me/achievements/completed",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/achievements/completed")]
async fn delete_user_achievements_completed(
    session: Session,
    ids: web::Json<Vec<i32>>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let mut complete = database::DbUserAchievementCompleted { username, id: 0 };

    for id in ids.0 {
        complete.id = id;

        database::delete_user_achievement_completed(&complete, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
