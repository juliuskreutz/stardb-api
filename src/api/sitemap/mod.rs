use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "sitemap")),
    paths(sitemap)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(sitemap);
}

const ROUTES: &[&str] = &[
    "https://stardb.gg",
    "https://stardb.gg/login",
    "https://stardb.gg/register",
    "https://stardb.gg/leaderboard",
    "https://stardb.gg/tier-list",
    "https://stardb.gg/achievement-tracker",
    "https://stardb.gg/profile-card-generator",
    "https://stardb.gg/privacy-policy",
    "https://stardb.gg/articles/",
    "https://stardb.gg/articles/daily-farm-route/",
    "https://stardb.gg/articles/free-stellar-jade-alerts/",
    "https://stardb.gg/articles/oneiric-shard-price/",
    "https://stardb.gg/articles/oneiric-shard-price-australia/",
    "https://stardb.gg/articles/oneiric-shard-price-brazil/",
    "https://stardb.gg/articles/oneiric-shard-price-canada/",
    "https://stardb.gg/articles/oneiric-shard-price-china/",
    "https://stardb.gg/articles/oneiric-shard-price-eu/",
    "https://stardb.gg/articles/oneiric-shard-price-india/",
    "https://stardb.gg/articles/oneiric-shard-price-indonesia/",
    "https://stardb.gg/articles/oneiric-shard-price-japan/",
    "https://stardb.gg/articles/oneiric-shard-price-kazakhstan/",
    "https://stardb.gg/articles/oneiric-shard-price-korea/",
    "https://stardb.gg/articles/oneiric-shard-price-malaysia/",
    "https://stardb.gg/articles/oneiric-shard-price-mexico/",
    "https://stardb.gg/articles/oneiric-shard-price-paraguay/",
    "https://stardb.gg/articles/oneiric-shard-price-phillipines/",
    "https://stardb.gg/articles/oneiric-shard-price-russia/",
    "https://stardb.gg/articles/oneiric-shard-price-singapore/",
    "https://stardb.gg/articles/oneiric-shard-price-taiwan/",
    "https://stardb.gg/articles/oneiric-shard-price-thailand/",
    "https://stardb.gg/articles/oneiric-shard-price-uk/",
    "https://stardb.gg/articles/oneiric-shard-price-us/",
    "https://stardb.gg/articles/oneiric-shard-price-vietnam/",
    "https://stardb.gg/api/help/",
];

#[derive(Serialize)]
#[allow(non_camel_case_types)]
struct urlset {
    #[serde(rename = "@xmlns")]
    xmlns: String,
    url: Vec<Url>,
}

#[derive(Serialize)]
struct Url {
    loc: String,
    lastmod: String,
}

#[utoipa::path(
    tag = "sitemap",
    get,
    path = "/api/sitemap",
    responses(
        (status = 200, description = "Sitemap"),
    )
)]
#[get("/api/sitemap")]
async fn sitemap(pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let lastmod = "2023-09-06";

    let mut urls = Vec::new();

    for route in ROUTES {
        let url = Url {
            loc: route.to_string(),
            lastmod: lastmod.to_string(),
        };

        urls.push(url);
    }

    for id in database::get_achievements_id(&pool).await? {
        let url = Url {
            loc: format!("https://stardb.gg/database/achievements/{id}"),
            lastmod: lastmod.to_string(),
        };

        urls.push(url);
    }

    let urlset = urlset {
        xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9".to_string(),
        url: urls,
    };

    let sitemap = r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string()
        + &quick_xml::se::to_string(&urlset)?;

    Ok(HttpResponse::Ok()
        .content_type("application/xml")
        .body(sitemap))
}
