#[macro_use]
extern crate tracing;

mod api;
mod database;
mod mihomo;
mod pg_session_store;
mod update;

use std::{env, fs};

use actix_files::Files;
use actix_session::{config::PersistentSession, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},
    middleware::Compress,
    web::Data,
    App, HttpServer,
};
use pg_session_store::PgSessionStore;
use sqlx::PgPool;
use utoipa_swagger_ui::SwaggerUi;

#[derive(
    Default,
    PartialEq,
    Eq,
    Hash,
    Clone,
    Copy,
    strum::Display,
    strum::EnumString,
    strum::EnumIter,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
    sqlx::Type,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "language", rename_all = "snake_case")]
enum Language {
    #[serde(alias = "zh-cn")]
    Chs,
    #[serde(alias = "zh-tw")]
    Cht,
    De,
    #[default]
    En,
    #[serde(alias = "es-es")]
    Es,
    Fr,
    Id,
    #[serde(alias = "ja")]
    Jp,
    #[serde(alias = "ko")]
    Kr,
    #[serde(alias = "pt-pt")]
    Pt,
    Ru,
    Th,
    Vi,
}

impl Language {
    pub fn name(&self) -> String {
        match self {
            Language::Chs => "简体中文",
            Language::Cht => "繁體中文",
            Language::De => "Deutsch",
            Language::En => "English",
            Language::Es => "Español",
            Language::Fr => "Français",
            Language::Id => "Bahasa Indonesia",
            Language::Jp => "日本語",
            Language::Kr => "한국어",
            Language::Pt => "Português",
            Language::Ru => "Русский",
            Language::Th => "ไทย",
            Language::Vi => "Tiếng Việt",
        }
        .to_string()
    }
}

#[derive(
    Clone,
    Copy,
    strum::Display,
    strum::EnumIter,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
    sqlx::Type,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "gacha_type", rename_all = "snake_case")]
enum GachaType {
    Standard,
    Departure,
    Special,
    Lc,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    tracing_subscriber::fmt::init();

    info!("Starting api!");

    let _ = fs::create_dir("mihomo");
    let _ = fs::create_dir("static");

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    update::achievements_percent(pool.clone()).await;
    update::books_percent(pool.clone()).await;
    update::community_tier_list(pool.clone()).await;
    update::dimbreath(pool.clone()).await;
    update::star_rail_res().await;
    update::scores().await;
    // update::warps_stats(pool.clone()).await;

    let pool_data = Data::new(pool.clone());

    let key = Key::from(&std::fs::read("session_key")?);

    let openapi = api::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .wrap(Compress::default())
            .wrap(if cfg!(debug_assertions) {
                SessionMiddleware::builder(PgSessionStore::new(pool.clone()), key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::weeks(4)))
                    .cookie_secure(false)
                    .build()
            } else {
                SessionMiddleware::builder(PgSessionStore::new(pool.clone()), key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::weeks(4)))
                    .build()
            })
            .service(Files::new("/static", "static").show_files_listing())
            .service(
                SwaggerUi::new("/api/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", openapi.clone()),
            )
            .configure(|sc| api::configure(sc, pool.clone()))
    })
    .bind(("localhost", 8000))?
    .run()
    .await?;

    info!("Stopping api!");

    std::process::exit(0)
}
