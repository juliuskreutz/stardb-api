mod id;

use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::ApiResult,
    database::{self, DbComplete},
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/achievements")),
    paths(get_user_achievements, put_user_achievements)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user_achievements)
        .service(put_user_achievements)
        .configure(id::configure);
}

#[utoipa::path(
    tag = "users/me/achievements",
    get,
    path = "/api/users/me/achievements",
    responses(
        (status = 200, description = "Achievement ids", body = Vec<i64>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/achievements")]
async fn get_user_achievements(
    session: Session,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let completed: Vec<_> = database::get_completed_by_username(&username, &pool)
        .await?
        .iter()
        .map(|c| c.id)
        .collect();

    Ok(HttpResponse::Ok().json(completed))
}

#[utoipa::path(
    tag = "users/me/achievements",
    put,
    path = "/api/users/me/achievements",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/achievements")]
async fn put_user_achievements(
    session: Session,
    ids: web::Json<Vec<i64>>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let mut complete = DbComplete { username, id: 0 };

    for id in ids.0 {
        complete.id = id;

        database::add_complete(&complete, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
