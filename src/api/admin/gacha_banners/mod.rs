use actix_session::Session;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "admin/gacha_banners")),
    paths(get_gacha_banners, create_gacha_banner, update_gacha_banner, delete_gacha_banner),
    components(schemas(
        database::gacha_banners::DbGachaBanner
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        .service(get_gacha_banners)
        .service(create_gacha_banner)
        .service(update_gacha_banner)
        .service(delete_gacha_banner);
}

#[utoipa::path(
    tag = "admin/gacha_banners",
    get,
    path = "/api/admin/gacha_banners",
    responses(
        (status = 200, description = "List of gacha banners", body = Vec<database::gacha_banners::DbGachaBanner>),
    ),
    security(("admin" = []))
)]
#[get("/api/admin/gacha_banners")]
async fn get_gacha_banners(session: Session, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let admin = database::admins::exists(&username, &pool).await?;
    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let banners = database::gacha_banners::get_all(&pool).await?;
    Ok(HttpResponse::Ok().json(banners))
}

#[utoipa::path(
    tag = "admin/gacha_banners",
    post,
    path = "/api/admin/gacha_banners",
    request_body = database::gacha_banners::DbGachaBanner,
    responses(
        (status = 201, description = "Created gacha banner", body = database::gacha_banners::DbGachaBanner)
    ),
    security(("admin" = []))
)]
#[post("/api/admin/gacha_banners")]
async fn create_gacha_banner(
    session: Session,
    pool: web::Data<PgPool>,
    banner: web::Json<database::gacha_banners::DbGachaBanner>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let admin = database::admins::exists(&username, &pool).await?;
    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let created_banner = database::gacha_banners::create(&banner, &pool).await?;
    Ok(HttpResponse::Created().json(created_banner))
}

#[utoipa::path(
    tag = "admin/gacha_banners",
    put,
    path = "/api/admin/gacha_banners/{id}",
    request_body = database::gacha_banners::DbGachaBanner,
    responses(
        (status = 200, description = "Updated gacha banner", body = database::gacha_banners::DbGachaBanner),
        (status = 404, description = "Gacha banner not found"),
    ),
    security(("admin" = []))
)]
#[put("/api/admin/gacha_banners/{id}")]
async fn update_gacha_banner(
    session: Session,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
    banner: web::Json<database::gacha_banners::DbGachaBanner>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let admin = database::admins::exists(&username, &pool).await?;
    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let mut banner = banner.into_inner();
    banner.id = *id;

    let updated_banner = database::gacha_banners::update(&banner, &pool).await?;
    if updated_banner.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }

    Ok(HttpResponse::Ok().json(updated_banner))
}

#[utoipa::path(
    tag = "admin/gacha_banners",
    delete,
    path = "/api/admin/gacha_banners/{id}",
    responses(
        (status = 204, description = "Gacha banner deleted"),
    ),
    security(("admin" = []))
)]
#[delete("/api/admin/gacha_banners/{id}")]
async fn delete_gacha_banner(
    session: Session,
    pool: web::Data<PgPool>,
    id: web::Path<i32>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let admin = database::admins::exists(&username, &pool).await?;
    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::gacha_banners::delete_by_id(*id, &pool).await?;
    Ok(HttpResponse::NoContent().finish())
}
