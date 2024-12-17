#[macro_use]
extern crate tracing;

mod api;
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
use futures::lock::Mutex;
use pg_session_store::PgSessionStore;
use sqlx::postgres::PgPoolOptions;
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
}

impl GachaType {
    pub fn id(self) -> i32 {
        match self {
            GachaType::Standard => 1,
            GachaType::Departure => 2,
            GachaType::Special => 11,
            GachaType::Lc => 12,
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

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    tracing_subscriber::fmt::init();

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

    update::achievements_percent::spawn(pool.clone()).await;
    update::zzz_achievements_percent::spawn(pool.clone()).await;
    update::gi_achievements_percent::spawn(pool.clone()).await;
    update::dimbreath::hsr::spawn(pool.clone()).await;
    update::dimbreath::zzz::spawn(pool.clone()).await;
    update::dimbreath::gi::spawn(pool.clone()).await;
    update::star_rail_res::spawn().await;
    //update::scores::spawn(pool.clone()).await;
    update::warps_stats::spawn(pool.clone()).await;
    update::signals_stats::spawn(pool.clone()).await;
    update::wishes_stats::spawn(pool.clone()).await;

    let pool_data = Data::new(pool.clone());

    let session_key = session_key()?;
    let signing_key = signing_key()?;
    let signing_key_data = web::Data::new(Mutex::new(signing_key));

    let openapi = api::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(web::JsonConfig::default().limit(5 * 1024 * 1024))
            .app_data(pool_data.clone())
            .app_data(signing_key_data.clone())
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
            .configure(|sc| api::configure(sc, pool.clone()))
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
        let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
        signing_key.write_pkcs8_pem_file(path, LineEnding::LF)?;
        signing_key
    })
}
