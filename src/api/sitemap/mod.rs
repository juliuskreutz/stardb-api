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

const LOCALIZED_ROUTES: &[&str] = &[
    "https://stardb.gg/%LANG%",
    "https://stardb.gg/%LANG%/achievement-tracker",
    "https://stardb.gg/%LANG%/book-tracker",
    "https://stardb.gg/%LANG%/free-stellar-jades",
    "https://stardb.gg/%LANG%/import",
    "https://stardb.gg/%LANG%/leaderboard",
    "https://stardb.gg/%LANG%/login",
    "https://stardb.gg/%LANG%/privacy-policy",
    "https://stardb.gg/%LANG%/register",
    "https://stardb.gg/%LANG%/request-token",
    "https://stardb.gg/%LANG%/warp-import",
    "https://stardb.gg/%LANG%/zzz/achievement-tracker",
    "https://stardb.gg/%LANG%/zzz/signal-tracker",
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
    let lastmod = "2024-06-21";

    let mut urls = Vec::new();

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

        for id in database::achievements::get_all_ids_shown(&pool).await? {
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

        for uid in database::mihomo::get_all_uids(&pool).await? {
            for path in ["overview", "characters", "collection"] {
                let mut links = Vec::new();

                for link_language in Language::iter() {
                    links.push(Link {
                        rel: "alternate".to_string(),
                        hreflang: link_language.to_string(),
                        href: format!("https://stardb.gg/{link_language}/profile/{uid}/{path}"),
                    });
                }

                let url = Url {
                    loc: format!("https://stardb.gg/{language}/profile/{uid}/{path}"),
                    lastmod: lastmod.to_string(),
                    links,
                };

                urls.push(url);
            }
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
