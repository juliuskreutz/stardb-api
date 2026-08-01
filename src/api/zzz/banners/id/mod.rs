use actix_session::Session;
use actix_web::{delete, get, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{zzz::banners::Banner, ApiResult},
    database,
    gacha::banner::validate_banner,
};

#[derive(OpenApi)]
#[openapi(tags((name="zzz/banners/{id}")), paths(get_banner, put_banner, delete_banner))]
struct ApiDoc;
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_banner)
        .service(put_banner)
        .service(delete_banner);
}

#[utoipa::path(tag="zzz/banners/{id}", get, path="/api/zzz/banners/{id}", responses((status=200, body=Banner)))]
#[get("/api/zzz/banners/{id}")]
async fn get_banner(id: web::Path<i32>, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    Ok(HttpResponse::Ok().json(Banner::from(
        database::zzz::banners::get_by_id(*id, &pool).await?,
    )))
}

#[derive(Deserialize, ToSchema)]
struct PutBanner {
    name: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    character: Option<i32>,
    character_gacha_type: Option<i32>,
    #[serde(alias = "weapon")]
    w_engine: Option<i32>,
    w_engine_gacha_type: Option<i32>,
    bangboo: Option<i32>,
    bangboo_gacha_type: Option<i32>,
}

#[utoipa::path(tag="zzz/banners/{id}", put, path="/api/zzz/banners/{id}", request_body=PutBanner, responses((status=200),(status=400),(status=403)))]
#[put("/api/zzz/banners/{id}")]
async fn put_banner(
    session: Session,
    id: web::Path<i32>,
    value: web::Json<PutBanner>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    if !database::admins::exists(&username, &pool).await? {
        return Ok(HttpResponse::Forbidden().finish());
    }
    if !validate_banner(
        value.start,
        value.end,
        &[
            (
                value.character.is_some(),
                value.character_gacha_type,
                &[2, 102],
            ),
            (
                value.w_engine.is_some(),
                value.w_engine_gacha_type,
                &[3, 103],
            ),
            (value.bangboo.is_some(), value.bangboo_gacha_type, &[5]),
        ],
    ) {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let banner = database::zzz::banners::DbBanner {
        id: *id,
        name: value.name.clone(),
        start: value.start,
        end: value.end,
        character: value.character,
        character_gacha_type: value.character_gacha_type,
        w_engine: value.w_engine,
        w_engine_gacha_type: value.w_engine_gacha_type,
        bangboo: value.bangboo,
        bangboo_gacha_type: value.bangboo_gacha_type,
    };
    database::zzz::banners::set(&banner, &**pool).await?;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(tag="zzz/banners/{id}", delete, path="/api/zzz/banners/{id}", responses((status=200),(status=403)))]
#[delete("/api/zzz/banners/{id}")]
async fn delete_banner(
    session: Session,
    id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    if !database::admins::exists(&username, &pool).await? {
        return Ok(HttpResponse::Forbidden().finish());
    }
    database::zzz::banners::delete_by_id(*id, &**pool).await?;
    Ok(HttpResponse::Ok().finish())
}
