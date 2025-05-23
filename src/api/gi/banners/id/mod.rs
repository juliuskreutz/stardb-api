use actix_session::Session;
use actix_web::{delete, get, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{gi::banners::GiBanner, ApiResult},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/banners/{id}")),
    paths(get_gi_banner, put_gi_banner, delete_gi_banner)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_gi_banner)
        .service(put_gi_banner)
        .service(delete_gi_banner);
}

#[utoipa::path(
    tag = "gi/banners/{id}",
    get,
    path = "/api/gi/banners/{id}",
    responses(
        (status = 200, description = "GiBanner", body = GiBanner),
    )
)]
#[get("/api/banners/{id}")]
async fn get_gi_banner(id: web::Path<i32>, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let banner: GiBanner = database::gi::banners::get_by_id(*id, &pool).await?.into();

    Ok(HttpResponse::Ok().json(banner))
}

#[derive(Deserialize, ToSchema)]
struct PutGiBanner {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    character: Option<i32>,
    weapon: Option<i32>,
}

#[utoipa::path(
    tag = "gi/banners/{id}",
    put,
    path = "/api/gi/banners/{id}",
    responses((status = 201)),
)]
#[put("/api/gi/banners/{id}")]
async fn put_gi_banner(
    session: Session,
    id: web::Path<i32>,
    banner: web::Json<PutGiBanner>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let admin = database::admins::exists(&username, &pool).await?;

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let db_banner = database::gi::banners::DbBanner {
        id: *id,
        start: banner.start,
        end: banner.end,
        character: banner.character,
        weapon: banner.weapon,
    };

    database::gi::banners::set(&db_banner, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "gi/banners/{id}",
    delete,
    path = "/api/gi/banners/{id}",
    responses((status = 200)),
)]
#[delete("/api/gi/banners/{id}")]
async fn delete_gi_banner(
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

    database::gi::banners::delete_by_id(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
