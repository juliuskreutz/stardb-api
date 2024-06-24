use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_web::{get, rt, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

lazy_static::lazy_static! {
    static ref CACHE: Mutex<Option<()>> = Mutex::new(None);
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
    CACHE.lock().unwrap().get_or_insert_with(|| cache(pool));

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

async fn update(pool: PgPool) -> anyhow::Result<()> {
    if std::path::Path::new("sitemap.xml").exists() {
        std::fs::remove_file("sitemap.xml")?;
    }

    let file = OpenOptions::new()
        .append(true)
        .create_new(true)
        .open("sitemap.xml")?;

    let mut writer = BufWriter::new(file);

    write!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    write!(
        writer,
        r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">"#
    )?;

    let lastmod = "2024-06-24";

    let achievement_ids = database::achievements::get_all_ids_shown(&pool).await?;
    let mihomo_uids = database::mihomo::get_all_uids(&pool).await?;

    let languages = [
        "de", "en", "es-es", "fr", "id", "ja", "ko", "pt-pt", "ru", "th", "vi", "zh-cn", "zh-tw",
    ];

    for language in languages {
        info!("{}", language);

        for route in LOCALIZED_ROUTES {
            write!(writer, "<url>")?;
            write!(writer, "<loc>{}</loc>", route.replace("%LANG%", language))?;
            write!(writer, "<lastmod>{lastmod}</lastmod>")?;

            for &link_language in &languages {
                write!(
                    writer,
                    r#"<xhtml:link rel="alternate" hreflang="{link_language}" href="{}" />"#,
                    route.replace("%LANG%", link_language)
                )?;
            }

            write!(writer, "</url>")?;
        }

        for id in &achievement_ids {
            write!(writer, "<url>")?;
            write!(
                writer,
                "<loc>https://stardb.gg/{language}/database/achievements/{id}</loc>"
            )?;
            write!(writer, "<lastmod>{lastmod}</lastmod>")?;

            for link_language in &languages {
                write!(
                    writer,
                    r#"<xhtml:link rel="alternate" hreflang="{link_language}" href="https://stardb.gg/{link_language}/database/achievements/{id}" />"#,
                )?;
            }

            write!(writer, "</url>")?;
        }

        for uid in &mihomo_uids {
            for path in ["overview", "characters", "collection"] {
                write!(writer, "<url>")?;
                write!(
                    writer,
                    "<loc>https://stardb.gg/{language}/profile/{uid}/{path}</loc>"
                )?;
                write!(writer, "<lastmod>{lastmod}</lastmod>")?;

                for link_language in &languages {
                    write!(
                        writer,
                        r#"<xhtml:link rel="alternate" hreflang="{link_language}" href="https://stardb.gg/{language}/profile/{uid}/{path}" />"#,
                    )?;
                }

                write!(writer, "</url>")?;
            }
        }
    }

    write!(writer, "</urlset>")?;

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
async fn sitemap() -> ApiResult<impl Responder> {
    Ok(HttpResponse::Ok()
        .content_type("application/xml")
        .body(std::fs::read("sitemap.xml")?))
}
