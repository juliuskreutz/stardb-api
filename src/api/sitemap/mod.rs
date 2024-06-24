use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_web::{get, rt, web, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

lazy_static::lazy_static! {
    static ref CACHE: Mutex<Option<()>> = Mutex::new(None);
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "sitemaps")),
    paths(sitemaps)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig, pool: PgPool) {
    CACHE.lock().unwrap().get_or_insert_with(|| cache(pool));

    cfg.service(sitemaps);
}

const LANGUAGES: &[&str] = &[
    "de", "en", "es-es", "fr", "id", "ja", "ko", "pt-pt", "ru", "th", "vi", "zh-cn", "zh-tw",
];

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

#[derive(serde::Serialize)]
#[serde(rename = "urlset")]
struct Urlset {
    #[serde(rename = "@xmlns")]
    xmlns: String,
    #[serde(rename = "@xmlns:xhtml")]
    xhtml: String,
    url: Vec<Url>,
}

#[derive(serde::Serialize)]
struct Url {
    loc: String,
    lastmod: String,
    #[serde(rename = "xhtml:link")]
    links: Vec<Link>,
}

#[derive(serde::Serialize)]
struct Link {
    #[serde(rename = "@rel")]
    rel: String,
    #[serde(rename = "@hreflang")]
    hreflang: String,
    #[serde(rename = "@href")]
    href: String,
}

#[derive(serde::Serialize)]
#[serde(rename = "sitemapindex")]
struct SitemapIndex {
    #[serde(rename = "@xmlns")]
    xmlns: String,
    sitemap: Vec<Sitemap>,
}

#[derive(serde::Serialize)]
struct Sitemap {
    loc: String,
}

pub fn cache(pool: PgPool) {
    rt::spawn(async move {
        let mut interval = rt::time::interval(Duration::from_secs(60 * 60 * 24));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(pool.clone()).await {
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

fn write_urls(count: usize, urls: Vec<Url>) -> anyhow::Result<()> {
    let file_name = format!("sitemaps/{count}.xml");

    let mut writer = r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string();

    let urlset = Urlset {
        xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9".to_string(),
        xhtml: "http://www.w3.org/1999/xhtml".to_string(),
        url: urls,
    };

    quick_xml::se::to_writer(&mut writer, &urlset)?;

    std::fs::write(file_name, writer)?;

    Ok(())
}

fn write_sitemap_index(count: usize) -> anyhow::Result<()> {
    let file_name = "sitemaps/index.xml";

    let mut writer = r#"<?xml version="1.0" encoding="UTF-8"?>"#.to_string();

    let mut sitemap = Vec::new();
    for i in 0..count {
        sitemap.push(Sitemap {
            loc: format!("https://stardb.gg/api/sitemaps/{i}"),
        })
    }

    let sitemap_index = SitemapIndex {
        xmlns: "http://www.sitemaps.org/schemas/sitemap/0.9".to_string(),
        sitemap,
    };

    quick_xml::se::to_writer(&mut writer, &sitemap_index)?;

    std::fs::write(file_name, writer)?;

    Ok(())
}

async fn update(pool: PgPool) -> anyhow::Result<()> {
    let lastmod = "2024-06-24";

    let achievement_ids = database::achievements::get_all_ids_shown(&pool).await?;
    let mihomo_uids = database::mihomo::get_all_uids(&pool).await?;

    let mut count = 0;

    let mut urls = Vec::new();

    for language in LANGUAGES {
        for route in LOCALIZED_ROUTES {
            let mut links = Vec::new();

            for &link_language in LANGUAGES {
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

            if urls.len() >= 3000 {
                write_urls(count, urls)?;
                count += 1;
                urls = Vec::new();
            }
        }

        for id in &achievement_ids {
            let mut links = Vec::new();

            for link_language in LANGUAGES {
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

            if urls.len() >= 3000 {
                write_urls(count, urls)?;
                count += 1;
                urls = Vec::new();
            }
        }

        for uid in &mihomo_uids {
            for path in ["overview", "characters", "collection"] {
                let mut links = Vec::new();

                for link_language in LANGUAGES {
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

                if urls.len() >= 3000 {
                    write_urls(count, urls)?;
                    count += 1;
                    urls = Vec::new();
                }
            }
        }
    }

    if !urls.is_empty() {
        write_urls(count, urls)?;
        count += 1;
    }

    write_sitemap_index(count)?;

    Ok(())
}

#[utoipa::path(
    tag = "sitemaps",
    get,
    path = "/api/sitemaps/{path}",
    responses(
        (status = 200, description = "sitemaps"),
    )
)]
#[get("/api/sitemaps/{path}")]
async fn sitemaps(path: web::Path<String>) -> ApiResult<impl Responder> {
    Ok(actix_files::NamedFile::open(format!(
        "sitemaps/{path}.xml"
    ))?)
}
