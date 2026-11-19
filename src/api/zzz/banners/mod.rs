//! Read endpoint and OpenAPI registration for ZZZ banner administration.

mod id;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(tags((name = "zzz/banners")), paths(get_banners), components(schemas(Banner)))]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
pub(super) struct Banner {
    id: i32,
    name: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    character: Option<i32>,
    character_gacha_type: Option<i32>,
    w_engine: Option<i32>,
    w_engine_gacha_type: Option<i32>,
    bangboo: Option<i32>,
    bangboo_gacha_type: Option<i32>,
}

impl From<database::zzz::banners::DbBanner> for Banner {
    fn from(value: database::zzz::banners::DbBanner) -> Self {
        Self {
            id: value.id,
            name: value.name,
            start: value.start,
            end: value.end,
            character: value.character,
            character_gacha_type: value.character_gacha_type,
            w_engine: value.w_engine,
            w_engine_gacha_type: value.w_engine_gacha_type,
            bangboo: value.bangboo,
            bangboo_gacha_type: value.bangboo_gacha_type,
        }
    }
}

/// Returns this route group's OpenAPI fragment.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut api = ApiDoc::openapi();
    api.merge(id::openapi());
    api
}
/// Registers ZZZ banner list and item routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_banners).configure(id::configure);
}

/// Lists every configured ZZZ banner.
#[utoipa::path(tag="zzz/banners", get, path="/api/zzz/banners", responses((status=200, body=Vec<Banner>)))]
#[get("/api/zzz/banners")]
async fn get_banners(pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let values: Vec<_> = database::zzz::banners::get_all(&pool)
        .await?
        .into_iter()
        .map(Banner::from)
        .collect();
    Ok(HttpResponse::Ok().json(values))
}

#[cfg(test)]
mod zzz_banner {
    #[test]
    fn routes() {
        let namespaces = [
            crate::api::banners::openapi(),
            crate::api::gi::openapi(),
            super::openapi(),
        ];
        let expected = [
            ["/api/banners", "/api/banners/{id}"],
            ["/api/gi/banners", "/api/gi/banners/{id}"],
            ["/api/zzz/banners", "/api/zzz/banners/{id}"],
        ];

        for (api, paths) in namespaces.into_iter().zip(expected) {
            for path in paths {
                assert!(
                    api.paths.paths.contains_key(path),
                    "missing OpenAPI path {path}"
                );
            }
        }
    }
}
