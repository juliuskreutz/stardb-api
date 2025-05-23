mod id;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/banners")),
    paths(get_gi_banners),
    components(schemas(
        GiBanner
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct GiBanner {
    id: i32,
    name: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    character: Option<i32>,
    weapon: Option<i32>,
}

impl From<database::gi::banners::DbBanner> for GiBanner {
    fn from(banner: database::gi::banners::DbBanner) -> Self {
        Self {
            id: banner.id,
            name: banner.name,
            start: banner.start,
            end: banner.end,
            character: banner.character,
            weapon: banner.weapon,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_gi_banners).configure(id::configure);
}

#[utoipa::path(
    tag = "gi/banners",
    get,
    path = "/api/gi/banners",
    responses(
        (status = 200, description = "[GiBanner]", body = Vec<GiBanner>),
    )
)]
#[get("/api/gi/banners")]
async fn get_gi_banners(pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let banners: Vec<_> = database::gi::banners::get_all(&pool)
        .await?
        .into_iter()
        .map(GiBanner::from)
        .collect();

    Ok(HttpResponse::Ok().json(banners))
}
