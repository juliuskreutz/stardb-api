use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_web::{
    get,
    rt::{self, Runtime},
    web, Responder,
};
use sqlx::PgPool;
use strum::IntoEnumIterator;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, Language};

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

const LASTMOD: &str = "2024-06-26";
const MAX_URLS: usize = 20000;

const LOCALIZED_ROUTES: &[&str] = &[
    "https://stardb.gg/%LANG%",
    "https://stardb.gg/%LANG%/achievement-tracker",
    "https://stardb.gg/%LANG%/import",
    "https://stardb.gg/%LANG%/leaderboard",
    "https://stardb.gg/%LANG%/login",
    "https://stardb.gg/%LANG%/privacy-policy",
    "https://stardb.gg/%LANG%/register",
    "https://stardb.gg/%LANG%/request-token",
    "https://stardb.gg/%LANG%/warp-import",
    "https://stardb.gg/%LANG%/warp-tracker",
    "https://stardb.gg/%LANG%/zzz/achievement-tracker",
    "https://stardb.gg/%LANG%/zzz/signal-tracker",
    "https://stardb.gg/%LANG%/zzz/signal-import",
    "https://stardb.gg/%LANG%/genshin/achievement-tracker",
    "https://stardb.gg/%LANG%/genshin/wish-tracker",
    "https://stardb.gg/%LANG%/genshin/wish-import",
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
    lastmod: String,
}

pub fn cache(pool: PgPool) {
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        let handle = rt.spawn(async move {
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

        rt.block_on(handle).unwrap();
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
            lastmod: LASTMOD.to_string(),
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
    let achievement_ids = database::achievements::get_all_ids_shown(&pool).await?;
    let zzz_achievement_ids = database::zzz::achievements::get_all_ids_shown(&pool).await?;
    let gi_achievement_ids = database::gi::achievements::get_all_ids_shown(&pool).await?;

    let mihomo_uids = database::mihomo::get_all_uids(&pool).await?;
    let warp_uids = database::warps::get_uids(&pool).await?;
    let signal_uids = database::zzz::signals::get_uids(&pool).await?;
    let wish_uids = database::gi::wishes::get_uids(&pool).await?;

    let mut count = 0;

    let mut urls = Vec::new();

    let languages: Vec<_> = Language::iter().map(|l| l.to_string()).collect();

    for uid in &mihomo_uids {
        for path in ["overview", "characters", "collection"] {
            let links = Vec::new();

            let url = Url {
                loc: format!("https://stardb.gg/en/profile/{uid}/{path}"),
                lastmod: LASTMOD.to_string(),
                links,
            };

            urls.push(url);

            if urls.len() >= MAX_URLS {
                write_urls(count, urls)?;
                count += 1;
                urls = Vec::new();
            }
        }
    }

    for language in &languages {
        for route in LOCALIZED_ROUTES {
            let mut links = Vec::new();

            for link_language in &languages {
                links.push(Link {
                    rel: "alternate".to_string(),
                    hreflang: link_language.to_string(),
                    href: route.replace("%LANG%", link_language),
                });
            }

            let url = Url {
                loc: route.replace("%LANG%", language),
                lastmod: LASTMOD.to_string(),
                links,
            };

            urls.push(url);

            if urls.len() >= MAX_URLS {
                write_urls(count, urls)?;
                count += 1;
                urls = Vec::new();
            }
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
                lastmod: LASTMOD.to_string(),
                links,
            };

            urls.push(url);

            if urls.len() >= MAX_URLS {
                write_urls(count, urls)?;
                count += 1;
                urls = Vec::new();
            }
        }

        for id in &zzz_achievement_ids {
            let mut links = Vec::new();

            for link_language in &languages {
                links.push(Link {
                    rel: "alternate".to_string(),
                    hreflang: link_language.to_string(),
                    href: format!(
                        "https://stardb.gg/{link_language}/zzz/database/achievements/{id}"
                    ),
                });
            }

            let url = Url {
                loc: format!("https://stardb.gg/{language}/zzz/database/achievements/{id}"),
                lastmod: LASTMOD.to_string(),
                links,
            };

            urls.push(url);

            if urls.len() >= MAX_URLS {
                write_urls(count, urls)?;
                count += 1;
                urls = Vec::new();
            }
        }

        for id in &gi_achievement_ids {
            let mut links = Vec::new();

            for link_language in &languages {
                links.push(Link {
                    rel: "alternate".to_string(),
                    hreflang: link_language.to_string(),
                    href: format!(
                        "https://stardb.gg/{link_language}/genshin/database/achievements/{id}"
                    ),
                });
            }

            let url = Url {
                loc: format!("https://stardb.gg/{language}/genshin/database/achievements/{id}"),
                lastmod: LASTMOD.to_string(),
                links,
            };

            urls.push(url);

            if urls.len() >= MAX_URLS {
                write_urls(count, urls)?;
                count += 1;
                urls = Vec::new();
            }
        }

        //TODO: Comment in when client emulator works
        //
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
        //            lastmod: LASTMOD.to_string(),
        //            links,
        //        };
        //
        //        urls.push(url);
        //
        //        if urls.len() >= MAX_URLS {
        //            write_urls(count, urls)?;
        //            count += 1;
        //            urls = Vec::new();
        //        }
        //    }
        //}

        for uid in &warp_uids {
            let mut links = Vec::new();

            for link_language in &languages {
                links.push(Link {
                    rel: "alternate".to_string(),
                    hreflang: link_language.to_string(),
                    href: format!("https://stardb.gg/{link_language}/warp-tracker/{uid}"),
                });
            }

            let url = Url {
                loc: format!("https://stardb.gg/{language}/warp-tracker/{uid}"),
                lastmod: LASTMOD.to_string(),
                links,
            };

            urls.push(url);

            if urls.len() >= MAX_URLS {
                write_urls(count, urls)?;
                count += 1;
                urls = Vec::new();
            }
        }

        for uid in &signal_uids {
            let mut links = Vec::new();

            for link_language in &languages {
                links.push(Link {
                    rel: "alternate".to_string(),
                    hreflang: link_language.to_string(),
                    href: format!("https://stardb.gg/{link_language}/zzz/signal-tracker/{uid}"),
                });
            }

            let url = Url {
                loc: format!("https://stardb.gg/{language}/zzz/signal-tracker/{uid}"),
                lastmod: LASTMOD.to_string(),
                links,
            };

            urls.push(url);

            if urls.len() >= MAX_URLS {
                write_urls(count, urls)?;
                count += 1;
                urls = Vec::new();
            }
        }

        for uid in &wish_uids {
            let mut links = Vec::new();

            for link_language in &languages {
                links.push(Link {
                    rel: "alternate".to_string(),
                    hreflang: link_language.to_string(),
                    href: format!("https://stardb.gg/{link_language}/genshin/wish-tracker/{uid}"),
                });
            }

            let url = Url {
                loc: format!("https://stardb.gg/{language}/genshin/wish-tracker/{uid}"),
                lastmod: LASTMOD.to_string(),
                links,
            };

            urls.push(url);

            if urls.len() >= MAX_URLS {
                write_urls(count, urls)?;
                count += 1;
                urls = Vec::new();
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
