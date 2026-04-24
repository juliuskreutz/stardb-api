use actix_session::Session;
use actix_web::{delete, get, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{zzz::banners::ZzzBanner, ApiResult},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "zzz/banners/{id}")),
    paths(get_zzz_banner, put_zzz_banner, delete_zzz_banner)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_zzz_banner)
        .service(put_zzz_banner)
        .service(delete_zzz_banner);
}

#[utoipa::path(
    tag = "zzz/banners/{id}",
    get,
    path = "/api/zzz/banners/{id}",
    responses(
        (status = 200, description = "ZzzBanner", body = ZzzBanner),
    )
)]
#[get("/api/zzz/banners/{id}")]
async fn get_zzz_banner(id: web::Path<i32>, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let banner: ZzzBanner = database::zzz::banners::get_by_id(*id, &pool).await?.into();

    Ok(HttpResponse::Ok().json(banner))
}

#[derive(Deserialize, ToSchema)]
struct PutZzzBanner {
    name: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    character: Option<i32>,
    #[serde(alias = "weapon")]
    w_engine: Option<i32>,
    bangboo: Option<i32>,
}

#[utoipa::path(
    tag = "zzz/banners/{id}",
    put,
    path = "/api/zzz/banners/{id}",
    responses(
        (status = 200),
        (status = 403),
    ),
)]
#[put("/api/zzz/banners/{id}")]
async fn put_zzz_banner(
    session: Session,
    id: web::Path<i32>,
    banner: web::Json<PutZzzBanner>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let admin = database::admins::exists(&username, &pool).await?;

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let db_banner = database::zzz::banners::DbBanner {
        id: *id,
        name: banner.name.clone(),
        start: banner.start,
        end: banner.end,
        character: banner.character,
        w_engine: banner.w_engine,
        bangboo: banner.bangboo,
    };

    database::zzz::banners::set(&db_banner, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "zzz/banners/{id}",
    delete,
    path = "/api/zzz/banners/{id}",
    responses(
        (status = 200),
        (status = 403),
    ),
)]
#[delete("/api/zzz/banners/{id}")]
async fn delete_zzz_banner(
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

    database::zzz::banners::delete_by_id(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
