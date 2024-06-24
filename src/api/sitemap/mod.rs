use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
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
    tags((name = "sitemap.xml")),
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

fn file_writer(file_name: &str) -> anyhow::Result<BufWriter<File>> {
    if std::path::Path::new(file_name).exists() {
        std::fs::remove_file(file_name)?;
    }

    Ok(BufWriter::new(
        OpenOptions::new()
            .append(true)
            .create_new(true)
            .open(file_name)?,
    ))
}

async fn update(pool: PgPool) -> anyhow::Result<()> {
    let _ = std::fs::remove_dir_all("sitemap");
    std::fs::create_dir("sitemap")?;

    let mut writer = file_writer("sitemap/index.xml")?;

    write!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    write!(
        writer,
        r#"<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#
    )?;
    for language in LANGUAGES {
        write!(writer, "<sitemap>")?;
        write!(
            writer,
            "<loc>https://stardb.gg/api/sitemap/{language}</loc>"
        )?;
        write!(writer, "</sitemap>")?;
    }
    write!(writer, "</sitemapindex>")?;

    let lastmod = "2024-06-24";

    let achievement_ids = database::achievements::get_all_ids_shown(&pool).await?;
    let mihomo_uids = database::mihomo::get_all_uids(&pool).await?;

    for language in LANGUAGES {
        let mut writer = file_writer(&format!("sitemap/{language}.xml"))?;

        write!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        write!(
            writer,
            r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">"#
        )?;

        for route in LOCALIZED_ROUTES {
            write!(writer, "<url>")?;
            write!(writer, "<loc>{}</loc>", route.replace("%LANG%", language))?;
            write!(writer, "<lastmod>{lastmod}</lastmod>")?;

            for &link_language in LANGUAGES {
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

            for link_language in LANGUAGES {
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

                for link_language in LANGUAGES {
                    write!(
                        writer,
                        r#"<xhtml:link rel="alternate" hreflang="{link_language}" href="https://stardb.gg/{language}/profile/{uid}/{path}" />"#,
                    )?;
                }

                write!(writer, "</url>")?;
            }
        }

        write!(writer, "</urlset>")?;
    }

    Ok(())
}

#[utoipa::path(
    tag = "sitemap",
    get,
    path = "/api/sitemap/{path}",
    responses(
        (status = 200, description = "sitemap"),
    )
)]
#[get("/api/sitemap/{path}")]
async fn sitemap(path: web::Path<String>) -> ApiResult<impl Responder> {
    Ok(actix_files::NamedFile::open(format!("sitemap/{path}.xml"))?)
}
