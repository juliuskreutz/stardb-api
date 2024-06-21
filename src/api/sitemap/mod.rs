use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_web::{get, rt, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

lazy_static::lazy_static! {
    static ref CACHE: Mutex<Option<web::Data<SitemapCache>>> = Mutex::new(None);
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "sitemap")),
    paths(sitemap)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig, pool: PgPool) {
    let data = CACHE
        .lock()
        .unwrap()
        .get_or_insert_with(|| cache(pool))
        .clone();

    cfg.service(sitemap).app_data(data);
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

#[derive(Default)]
pub struct SitemapCache {
    sitemap: futures::lock::Mutex<String>,
}

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

pub fn cache(pool: PgPool) -> web::Data<SitemapCache> {
    let sitemap_data = web::Data::new(SitemapCache::default());

    {
        let sitemap_cache = sitemap_data.clone();

        rt::spawn(async move {
            let mut interval = rt::time::interval(Duration::from_secs(60 * 60 * 24));

            rt::time::sleep(Duration::from_secs(60)).await;

            loop {
                interval.tick().await;

                let start = Instant::now();

                if let Err(e) = update(&sitemap_cache, pool.clone()).await {
                    error!(
                        "Sitemap update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    info!(
                        "Sitemap update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });
    }

    sitemap_data
}

async fn update(sitemap_cache: &web::Data<SitemapCache>, pool: PgPool) -> anyhow::Result<()> {
    let lastmod = "2024-06-21";

    let mut urls = Vec::new();

    let achievement_ids = database::achievements::get_all_ids_shown(&pool).await?;
    //let mihomo_uids = database::mihomo::get_all_uids(&pool).await?;

    let languages = [
        "de", "en", "es-es", "fr", "id", "ja", "ko", "pt-pt", "ru", "th", "vi", "zh-cn", "zh-tw",
    ];

    for language in &languages {
        rt::task::yield_now().await;

        info!("{}", language);

        for route in LOCALIZED_ROUTES {
            let mut links = Vec::new();

            for &link_language in &languages {
                links.push(Link {
                    rel: "alternate".to_string(),
                    hreflang: link_language.to_string(),
                    href: route.replace("%LANG%", link_language),
                });
            }

            let url = Url {
                loc: route.replace("%LANG%", language),
                lastmod: lastmod.to_string(),
                links,
            };

            urls.push(url);
        }

        for id in &achievement_ids {
            let mut links = Vec::new();

            for link_language in &languages {
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

        //for uid in &mihomo_uids {
        //    for path in ["overview", "characters", "collection"] {
        //        let mut links = Vec::new();
        //
        //        for link_language in &languages {
        //            links.push(Link {
        //                rel: "alternate".to_string(),
        //                hreflang: link_language.to_string(),
        //                href: format!("https://stardb.gg/{link_language}/profile/{uid}/{path}"),
        //            });
        //        }
        //
        //        let url = Url {
        //            loc: format!("https://stardb.gg/{language}/profile/{uid}/{path}"),
        //            lastmod: lastmod.to_string(),
        //            links,
        //        };
        //
        //        urls.push(url);
        //    }
        //}
    }

    let urlset = Urlset {
        xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9".to_string(),
        xhtml: "http://www.w3.org/1999/xhtml".to_string(),
        url: urls,
    };

    *sitemap_cache.sitemap.lock().await = r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string()
        + &quick_xml::se::to_string(&urlset)?;

    Ok(())
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
async fn sitemap(sitemap_cache: web::Data<SitemapCache>) -> ApiResult<impl Responder> {
    Ok(HttpResponse::Ok()
        .content_type("application/xml")
        .body(sitemap_cache.sitemap.lock().await.clone()))
}

