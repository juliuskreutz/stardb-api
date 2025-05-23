use actix_session::Session;
use actix_web::{delete, get, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{banners::Banner, ApiResult},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "banners/{id}")),
    paths(get_banner, put_banner, delete_banner)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_banner)
        .service(put_banner)
        .service(delete_banner);
}

#[utoipa::path(
    tag = "banners/{id}",
    get,
    path = "/api/banners/{id}",
    responses(
        (status = 200, description = "Banner", body = Banner),
    )
)]
#[get("/api/banners/{id}")]
async fn get_banner(id: web::Path<i32>, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let banner: Banner = database::banners::get_by_id(*id, &pool).await?.into();

    Ok(HttpResponse::Ok().json(banner))
}

#[derive(Deserialize, ToSchema)]
struct PutBanner {
    name: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    character: Option<i32>,
    light_cone: Option<i32>,
}

#[utoipa::path(
    tag = "banners/{id}",
    put,
    path = "/api/banners/{id}",
    responses(
        (status = 201),
        (status = 403),
    ),
)]
#[put("/api/banners/{id}")]
async fn put_banner(
    session: Session,
    id: web::Path<i32>,
    banner: web::Json<PutBanner>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let admin = database::admins::exists(&username, &pool).await?;

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let db_banner = database::banners::DbBanner {
        id: *id,
        name: banner.name.clone(),
        start: banner.start,
        end: banner.end,
        character: banner.character,
        light_cone: banner.light_cone,
    };

    database::banners::set(&db_banner, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "banners/{id}",
    delete,
    path = "/api/banners/{id}",
    responses(
        (status = 200),
        (status = 403),
    ),
)]
#[delete("/api/banners/{id}")]
async fn delete_banner(
    session: Session,
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let admin = database::admins::exists(&username, &pool).await?;

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::banners::delete_by_id(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
