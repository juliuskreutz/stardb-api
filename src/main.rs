mod api;
mod database;
mod mihomo;
mod update;

use std::{collections::HashMap, fs};

use actix_files::Files;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{time::Duration, Key},
    web::Data,
    App, HttpServer,
};
use convert_case::{Case, Casing};
use futures::lock::Mutex;
use sqlx::PgPool;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

trait ToTag {
    fn to_tag(&self) -> String;
}

impl<T: AsRef<str>> ToTag for T {
    fn to_tag(&self) -> String {
        self.as_ref()
            .replace(|c: char| !c.is_alphanumeric(), " ")
            .to_case(Case::Kebab)
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();

    dotenv::dotenv()?;

    let _ = fs::create_dir("mihomo");

    let pool = PgPool::connect(&dotenv::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;

    update::dimbreath(pool.clone()).await;
    update::verifications(pool.clone()).await;
    update::scores().await;

    let tokens_data = Data::new(Mutex::new(HashMap::<Uuid, String>::new()));
    let pool_data = Data::new(pool.clone());

    let key = Key::generate();

    let openapi = api::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(tokens_data.clone())
            .app_data(pool_data.clone())
            .wrap(if cfg!(debug_assertions) {
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::weeks(4)))
                    .cookie_secure(false)
                    .build()
            } else {
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .session_lifecycle(PersistentSession::default().session_ttl(Duration::weeks(4)))
                    .build()
            })
            .service(Files::new("/static", "static").show_files_listing())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi.clone()),
            )
            .configure(|cfg| api::configure(cfg, pool.clone()))
    })
    .bind(("localhost", 8000))?
    .run()
    .await?;

    Ok(())
}
