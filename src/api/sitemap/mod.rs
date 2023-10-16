use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use strum::IntoEnumIterator;
use utoipa::OpenApi;

use crate::{
    api::{ApiResult, Language},
    database,
};

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
    "https://stardb.gg/warps/",
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

const LOCALIZED_ROUTES: &[&str] = &[
    "https://stardb.gg/%LANG%",
    "https://stardb.gg/%LANG%/login",
    "https://stardb.gg/%LANG%/register",
    "https://stardb.gg/%LANG%/leaderboard",
    "https://stardb.gg/%LANG%/tier-list",
    "https://stardb.gg/%LANG%/achievement-tracker",
    "https://stardb.gg/%LANG%/profile-card-generator",
    "https://stardb.gg/%LANG%/privacy-policy",
];

#[derive(Serialize)]
#[serde(rename = "urlset")]
struct Urlset {
    #[serde(rename = "@xmlns")]
    xmlns: String,
    #[serde(rename = "@xmlns:xhtml")]
    xhtml: String,
    url: Vec<Url>,
}

#[derive(Serialize)]
struct Url {
    loc: String,
    lastmod: String,
    #[serde(rename = "xhtml:link")]
    links: Vec<Link>,
}

#[derive(Serialize)]
struct Link {
    #[serde(rename = "@rel")]
    rel: String,
    #[serde(rename = "@hreflang")]
    hreflang: String,
    #[serde(rename = "@href")]
    href: String,
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
    let lastmod = "2023-10-16";

    let mut urls = Vec::new();

    for route in ROUTES {
        let url = Url {
            loc: route.to_string(),
            lastmod: lastmod.to_string(),
            links: Vec::new(),
        };

        urls.push(url);
    }

    for language in Language::iter() {
        for route in LOCALIZED_ROUTES {
            let mut links = Vec::new();

            for link_language in Language::iter() {
                links.push(Link {
                    rel: "alternate".to_string(),
                    hreflang: link_language.to_string(),
                    href: route.replace("%LANG%", &link_language.to_string()),
                });
            }

            let url = Url {
                loc: route.replace("%LANG%", &language.to_string()),
                lastmod: lastmod.to_string(),
                links,
            };

            urls.push(url);
        }

        for id in database::get_achievements_id(&pool).await? {
            let mut links = Vec::new();

            for link_language in Language::iter() {
                links.push(Link {
                    rel: "alternate".to_string(),
                    hreflang: link_language.to_string(),
                    href: format!("https://stardb.gg/{link_language}/database/achievements/{id}"),
                });
            }

            let url = Url {
                loc: format!("https://stardb.gg/{language}/database/achievements/{id}"),
                lastmod: lastmod.to_string(),
                links,
            };

            urls.push(url);
        }
    }

    let urlset = Urlset {
        xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9".to_string(),
        xhtml: "http://www.w3.org/1999/xhtml".to_string(),
        url: urls,
    };

    let sitemap = r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string()
        + &quick_xml::se::to_string(&urlset)?;

    Ok(HttpResponse::Ok()
        .content_type("application/xml")
        .body(sitemap))
}
