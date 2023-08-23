mod api;
mod database;
mod mihomo;
mod pg_session_store;
mod update;

use std::{
    collections::HashMap,
    env,
    fs::{self, File},
};

use actix_files::Files;
use actix_session::{config::PersistentSession, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},
    middleware::Compress,
    web::Data,
    App, HttpServer,
};
use futures::lock::Mutex;
use sqlx::PgPool;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use pg_session_store::PgSessionStore;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;

    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Pipe(Box::new(
            File::options().append(true).create(true).open("log.log")?,
        )))
        .init();

    log::info!("Starting api!");

    let _ = fs::create_dir("mihomo");

    let pool = PgPool::connect(&env::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    update::achievements_percent(pool.clone()).await;
    update::community_tier_list(pool.clone()).await;
    update::dimbreath(pool.clone()).await;
    update::verifications(pool.clone()).await;
    update::scores().await;

    let pool_data = Data::new(pool.clone());
    let tokens_data = Data::new(Mutex::new(HashMap::<Uuid, String>::new()));
    //FIXME: This is ugly as hell
    let achievement_tracker_cache_data = api::cache_achievement_tracker(pool.clone());

    let key = Key::from(&std::fs::read("session_key")?);

    let openapi = api::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(tokens_data.clone())
            .app_data(pool_data.clone())
            .app_data(achievement_tracker_cache_data.clone())
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
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi.clone()),
            )
            .configure(api::configure)
    })
    .bind(("localhost", 8000))?
    .run()
    .await?;

    Ok(())
}
