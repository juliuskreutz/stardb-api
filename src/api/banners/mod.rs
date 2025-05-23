mod id;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "banners")),
    paths(get_banners),
    components(schemas(
        Banner
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct Banner {
    id: i32,
    name: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    character: Option<i32>,
    light_cone: Option<i32>,
}

impl From<database::banners::DbBanner> for Banner {
    fn from(banner: database::banners::DbBanner) -> Self {
        Self {
            id: banner.id,
            name: banner.name,
            start: banner.start,
            end: banner.end,
            character: banner.character,
            light_cone: banner.light_cone,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_banners).configure(id::configure);
}

#[utoipa::path(
    tag = "banners",
    get,
    path = "/api/banners",
    responses(
        (status = 200, description = "[Banner]", body = Vec<Banner>),
    )
)]
#[get("/api/banners")]
async fn get_banners(pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let banners: Vec<_> = database::banners::get_all(&pool)
        .await?
        .into_iter()
        .map(Banner::from)
        .collect();

    Ok(HttpResponse::Ok().json(banners))
}
