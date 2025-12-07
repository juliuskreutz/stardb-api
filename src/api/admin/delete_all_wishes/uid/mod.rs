use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "admin/delete-all-wishes/{uid}")),
    paths(post_delete_all_wishes),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_delete_all_wishes);
}

#[utoipa::path(
    tag = "admin/delete-all-wishes/{uid}",
    post,
    path = "/api/admin/delete-all-wishes/{uid}",
    responses(
        (status = 200, description = "Wishes delete"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[post("/api/admin/delete-all-wishes/{uid}")]
async fn post_delete_all_wishes(
    session: Session,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let admin = database::admins::exists(&username, &pool).await?;

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let uid = *uid;

    database::gi::wishes::beginner::delete_all(uid, &pool).await?;
    database::gi::wishes::standard::delete_all(uid, &pool).await?;
    database::gi::wishes::character::delete_all(uid, &pool).await?;
    database::gi::wishes::weapon::delete_all(uid, &pool).await?;
    database::gi::wishes::chronicled::delete_all(uid, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
