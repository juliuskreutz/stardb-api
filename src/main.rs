mod api;
mod database;
mod mihomo;
mod update;

use std::{collections::HashMap, fs, sync::Mutex};

use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web::Data, App, HttpServer};
use convert_case::{Case, Casing};
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

    update::achievements(pool.clone()).await;
    update::verifications(pool.clone()).await;
    update::scores().await;

    let password_resets = Data::new(Mutex::new(HashMap::<Uuid, String>::new()));
    let pool = Data::new(pool);

    let key = Key::generate();

    let openapi = api::openapi();

    HttpServer::new(move || {
        App::new()
            .app_data(password_resets.clone())
            .app_data(pool.clone())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                key.clone(),
            ))
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
