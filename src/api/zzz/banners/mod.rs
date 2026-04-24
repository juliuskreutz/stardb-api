mod id;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "zzz/banners")),
    paths(get_zzz_banners),
    components(schemas(
        ZzzBanner
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct ZzzBanner {
    id: i32,
    name: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    character: Option<i32>,
    w_engine: Option<i32>,
    bangboo: Option<i32>,
}

impl From<database::zzz::banners::DbBanner> for ZzzBanner {
    fn from(banner: database::zzz::banners::DbBanner) -> Self {
        Self {
            id: banner.id,
            name: banner.name,
            start: banner.start,
            end: banner.end,
            character: banner.character,
            w_engine: banner.w_engine,
            bangboo: banner.bangboo,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_zzz_banners).configure(id::configure);
}

#[utoipa::path(
    tag = "zzz/banners",
    get,
    path = "/api/zzz/banners",
    responses(
        (status = 200, description = "[ZzzBanner]", body = Vec<ZzzBanner>),
    )
)]
#[get("/api/zzz/banners")]
async fn get_zzz_banners(pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let banners: Vec<_> = database::zzz::banners::get_all(&pool)
        .await?
        .into_iter()
        .map(ZzzBanner::from)
        .collect();

    Ok(HttpResponse::Ok().json(banners))
}
