#[macro_use]
extern crate tracing;

mod api;
mod app_config;
mod database;
mod mihomo;
mod pg_session_store;
mod update;

use std::{env, fs, path::Path};

use actix_cors::Cors;
use actix_files::Files;
use actix_session::{config::PersistentSession, SessionMiddleware};
use actix_web::{
    cookie::time::Duration,
    middleware::Compress,
    web::{self, Data},
    App, HttpServer,
};
use ed25519_dalek::SecretKey;
use futures::lock::Mutex;
use pg_session_store::PgSessionStore;
use rand::RngCore;
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use tracing_subscriber::prelude::*;
use sentry_tracing::EventFilter;
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
)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
enum Language {
    ZhCn,
    ZhTw,
    De,
    #[default]
    En,
    EsEs,
    Fr,
    Id,
    Ja,
    Ko,
    PtPt,
    Ru,
    Th,
    Vi,
}

impl Language {
    pub fn name(&self) -> String {
        match self {
            Language::ZhCn => "简体中文",
            Language::ZhTw => "繁體中文",
            Language::De => "Deutsch",
            Language::En => "English",
            Language::EsEs => "Español",
            Language::Fr => "Français",
            Language::Id => "Bahasa Indonesia",
            Language::Ja => "日本語",
            Language::Ko => "한국어",
            Language::PtPt => "Português",
            Language::Ru => "Русский",
            Language::Th => "ไทย",
            Language::Vi => "Tiếng Việt",
        }
        .to_string()
    }

    pub fn mihomo(&self) -> String {
        match self {
            Language::ZhCn => "chs",
            Language::ZhTw => "cht",
            Language::De => "de",
            Language::En => "en",
            Language::EsEs => "es",
            Language::Fr => "fr",
            Language::Id => "id",
            Language::Ja => "jp",
            Language::Ko => "kr",
            Language::PtPt => "pt",
            Language::Ru => "ru",
            Language::Th => "th",
            Language::Vi => "vi",
        }
        .to_string()
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum GachaType {
    Standard,
    Departure,
    Special,
    Lc,
    Collab,
    CollabLc,
}

impl GachaType {
    pub fn id(self) -> i32 {
        match self {
            GachaType::Standard => 1,
            GachaType::Departure => 2,
            GachaType::Special => 11,
            GachaType::Lc => 12,
            GachaType::Collab => 21,
            GachaType::CollabLc => 22,
        }
    }
}

#[derive(
    Clone,
    Copy,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum ZzzGachaType {
    Standard,
    Special,
    WEngine,
    Bangboo,
    ExclusiveRescreening,
    WEngineReverberation,
}

impl ZzzGachaType {
    pub fn id(self) -> i32 {
        match self {
            ZzzGachaType::Standard => 1,
            ZzzGachaType::Special => 2,
            ZzzGachaType::WEngine => 3,
            ZzzGachaType::Bangboo => 5,
            ZzzGachaType::ExclusiveRescreening => 102,
            ZzzGachaType::WEngineReverberation => 103,
        }
    }

    pub fn old_id(self) -> i32 {
        match self {
            ZzzGachaType::Standard => 1001,
            ZzzGachaType::Special => 2001,
            ZzzGachaType::WEngine => 3001,
            ZzzGachaType::Bangboo => 5001,
            ZzzGachaType::ExclusiveRescreening => 12001,
            ZzzGachaType::WEngineReverberation => 13001,
        }
    }
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    strum::Display,
    strum::EnumIter,
    strum::EnumString,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum GiGachaType {
    Beginner,
    Standard,
    Character,
    Weapon,
    Chronicled,
}

#[derive(
    Clone,
    Copy,
    strum::Display,
    strum::EnumString,
    serde::Serialize,
    serde::Deserialize,
    utoipa::ToSchema,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

// Manual runtime setup (instead of #[actix_web::main]) so Sentry inits before the actix runtime —
// the guard must outlive main and the panic handler must be installed pre-runtime.
fn main() -> anyhow::Result<()> {
    let dotenv_path = dotenv::dotenv().ok();
    if let Some(path) = dotenv_path {
        tracing::debug!("Loaded .env file from: {:?}", path);
    } else {
        tracing::debug!("No .env file loaded");
    }

    // Sentry only activates when SENTRY_DSN is set. No DSN = no-op, guard is None.
    // Guard flushes pending events on drop at the end of main.
    let _sentry_guard = env::var("SENTRY_DSN").ok().map(|dsn| {
        sentry::init((
            dsn,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                environment: Some(
                    if cfg!(debug_assertions) {
                        "development"
                    } else {
                        "production"
                    }
                    .into(),
                ),
                traces_sample_rate: 0.0,
                attach_stacktrace: true,
                ..Default::default()
            },
        ))
    });

    // sentry_tracing layer forwards `error!` events (and panics) to Sentry.
    // Inert if Sentry wasn't initialized above.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with(
            sentry_tracing::layer().event_filter(|md| match *md.level() {
                tracing::Level::ERROR => EventFilter::Event,
                tracing::Level::WARN
                | tracing::Level::INFO
                | tracing::Level::DEBUG => EventFilter::Breadcrumb,
                tracing::Level::TRACE => EventFilter::Ignore,
            }),
        )
        .init();

    actix_web::rt::System::new().block_on(async_main())
}

async fn async_main() -> anyhow::Result<()> {
    let app_config = load_app_config()?;
    info!("Starting api!");

    let _ = fs::create_dir("mihomo");
    let _ = fs::create_dir("dimbreath");
    let _ = fs::create_dir("static");
    let _ = fs::create_dir("cache");
    let _ = fs::remove_dir_all("sitemaps");
    let _ = fs::create_dir("sitemaps");

    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect(&env::var("DATABASE_URL")?)
        .await?;
    sqlx::migrate!().run(&pool).await?;

    if app_config.enable_update_hsr_achievements_percent {
        update::achievements_percent::spawn(pool.clone()).await;
    }
    if app_config.enable_update_zzz_achievements_percent {
        update::zzz_achievements_percent::spawn(pool.clone()).await;
    }
    if app_config.enable_update_gi_achievements_percent {
        update::gi_achievements_percent::spawn(pool.clone()).await;
    }
    if app_config.enable_update_dimbreath_hsr {
        update::dimbreath::hsr::spawn(pool.clone()).await;
    }
    if app_config.enable_update_dimbreath_zzz {
        update::dimbreath::zzz::spawn(pool.clone()).await;
    }
    if app_config.enable_update_dimbreath_gi {
        update::dimbreath::gi::spawn(pool.clone()).await;
    }
    if app_config.enable_update_star_rail_res {
        update::star_rail_res::spawn().await;
    }
    if app_config.enable_update_scores {
        update::scores::spawn(pool.clone()).await;
    }
    if app_config.enable_update_warps_stats {
        update::warps_stats::spawn(pool.clone()).await;
    }
    if app_config.enable_update_signals_stats {
        update::signals_stats::spawn(pool.clone()).await;
    }
    if app_config.enable_update_wishes_stats {
        update::wishes_stats::spawn(pool.clone()).await;
    }

    let session_key = session_key()?;
    let signing_key = signing_key()?;

    let signing_key_data = web::Data::new(Mutex::new(signing_key));
    let pool_data = Data::new(pool.clone());
    let app_config_data = Data::new(app_config.clone());

    let openapi = api::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(web::JsonConfig::default().limit(5 * 1024 * 1024))
            .app_data(pool_data.clone())
            .app_data(signing_key_data.clone())
            .app_data(app_config_data.clone())
            // Captures request context + errors per-request. No-op if no SENTRY_DSN.
            .wrap(sentry_actix::Sentry::new())
            .wrap(Cors::permissive())
            .wrap(Compress::default())
            .wrap(if cfg!(debug_assertions) {
                SessionMiddleware::builder(PgSessionStore::new(pool.clone()), session_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::weeks(4)))
                    .cookie_secure(false)
                    .build()
            } else {
                SessionMiddleware::builder(PgSessionStore::new(pool.clone()), session_key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::weeks(4)))
                    .build()
            })
            .service(Files::new("/api/static", "static").show_files_listing())
            .service(
                SwaggerUi::new("/api/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", openapi.clone()),
            )
            .configure(|sc| api::configure(sc, pool.clone(), app_config_data.clone()))
    })
    .bind(("localhost", 8000))?
    .run()
    .await?;

    info!("Stopping api!");

    std::process::exit(0)
}

fn session_key() -> anyhow::Result<actix_web::cookie::Key> {
    use actix_web::cookie::Key;
    use base64::{prelude::BASE64_STANDARD, Engine};

    let key = if let Ok(bytes) = std::fs::read("session_key") {
        let key_bytes = BASE64_STANDARD.decode(bytes)?;

        Key::from(&key_bytes)
    } else {
        let key = Key::generate();
        let key_bytes = key.master();
        let bytes = BASE64_STANDARD.encode(key_bytes);
        std::fs::write("session_key", bytes)?;

        key
    };

    Ok(key)
}

fn signing_key() -> anyhow::Result<ed25519_dalek::SigningKey> {
    use ed25519_dalek::{
        pkcs8::{spki::der::pem::LineEnding, DecodePrivateKey, EncodePrivateKey},
        SigningKey,
    };

    let path = Path::new("id_ed25519_sign");

    Ok(if path.exists() {
        SigningKey::read_pkcs8_pem_file(path)?
    } else {
        let mut secret_key = SecretKey::default();
        rand::rng().fill_bytes(&mut secret_key);
        let signing_key = SigningKey::from_bytes(&secret_key);
        signing_key.write_pkcs8_pem_file(path, LineEnding::LF)?;
        signing_key
    })
}

fn load_app_config() -> anyhow::Result<Arc<app_config::AppConfig>> {
    let config = envy::from_env::<app_config::AppConfig>()?;
    tracing::debug!("AppConfig loaded: {:#?}", config);
    Ok(Arc::new(config))
}
