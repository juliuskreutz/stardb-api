use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/gi/achievements/favorites/{id}")),
    paths(put_gi_user_achievement_favorites, delete_gi_user_achievement_favorites)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_gi_user_achievement_favorites)
        .service(delete_gi_user_achievement_favorites);
}

#[utoipa::path(
    tag = "users/me/gi/achievements/favorites/{id}",
    put,
    path = "/api/users/me/gi/achievements/favorites/{id}",
    responses(
        (status = 200, description = "Successful add of the achievement"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/gi/achievements/favorites/{id}")]
async fn put_gi_user_achievement_favorites(
    session: Session,
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;
    let favorite =
        database::gi::users_achievements_favorites::DbUserAchievementFavorite { username, id };
    database::gi::users_achievements_favorites::add(&favorite, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/gi/achievements/favorites/{id}",
    delete,
    path = "/api/users/me/gi/achievements/favorites/{id}",
    responses(
        (status = 200, description = "Successful delete of the achievement"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/gi/achievements/favorites/{id}")]
async fn delete_gi_user_achievement_favorites(
    session: Session,
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let favorite =
        database::gi::users_achievements_favorites::DbUserAchievementFavorite { username, id };

    database::gi::users_achievements_favorites::delete(&favorite, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
