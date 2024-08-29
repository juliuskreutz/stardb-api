use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "admin/delete-unofficial-warps/{uid}")),
    paths(post_delete_unofficial_warps),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_delete_unofficial_warps);
}

#[utoipa::path(
    tag = "admin/delete-unofficial-warps/{uid}",
    post,
    path = "/api/admin/delete-unofficial-warps/{uid}",
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[post("/api/admin/delete-unofficial-warps/{uid}")]
async fn post_delete_unofficial_warps(
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

    database::warps::departure::delete_unofficial(uid, &pool).await?;
    database::warps::standard::delete_unofficial(uid, &pool).await?;
    database::warps::special::delete_unofficial(uid, &pool).await?;
    database::warps::lc::delete_unofficial(uid, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
