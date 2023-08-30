use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/achievements/{id}")),
    paths(put_user_achievement, delete_user_achievement)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(put_user_achievement)
        .service(delete_user_achievement);
}

#[utoipa::path(
    tag = "users/me/achievements/{id}",
    put,
    path = "/api/users/me/achievements/{id}",
    responses(
        (status = 200, description = "Successful add of the achievement"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/achievements/{id}")]
async fn put_user_achievement(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let forbidden = [
        4072605, 4072606, 4072607, 4092635, 4092637, 4092621, 4092636, 4092638, 4092639, 4092640,
        4092641, 4092633, 4092634, 4092601, 4092603, 4092604, 4092605, 4092606, 4092607, 4092608,
        4092609, 4092610, 4092611, 4092612, 4092613, 4092614, 4092615, 4092616, 4092617, 4092618,
        4092619, 4092620, 4092622, 4092623, 4092624, 4092625, 4092626, 4092627, 4092628, 4092629,
        4092630, 4092631, 4092632, // Sim U
        4052615, 4052616, // 1.3 Companion Missions
        4072601, 4072611, 4072612, 4070904, 4070910, 4092602, 4070915, // Fu Xuan Banner
        4020203, 4082301, // Not 1.3
    ];

    if !forbidden.contains(&id) {
        let db_complete = database::DbUserAchievement { username, id };

        database::add_user_achievement(&db_complete, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/achievements/{id}",
    delete,
    path = "/api/users/me/achievements/{id}",
    responses(
        (status = 200, description = "Successful delete of the achievement"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/achievements/{id}")]
async fn delete_user_achievement(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let db_complete = database::DbUserAchievement { username, id };

    database::delete_user_achievement(&db_complete, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
