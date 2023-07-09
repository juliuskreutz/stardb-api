mod api;
mod database;
mod mihomo;
mod update;

use actix_files::Files;
use rand::Rng;
use std::{collections::HashMap, fs, sync::Mutex};

use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[derive(OpenApi)]
#[openapi(
    paths(
        api::scores::get_scores_achievement,
        api::scores::get_score_achievement,
        api::scores::put_score_achievement,
        api::scores::damage::get_scores_damage,
        api::scores::damage::get_score_damage,
        api::scores::damage::put_score_damage,
        api::scores::heal::get_scores_heal,
        api::scores::heal::get_score_heal,
        api::scores::heal::put_score_heal,
        api::scores::shield::get_scores_shield,
        api::scores::shield::get_score_shield,
        api::scores::shield::put_score_shield,
        api::users::login,
        api::users::register,
        api::users::logout,
        api::users::request_token,
        api::users::get_me,
        api::users::put_email,
        api::users::put_password,
        api::achievements::get_achievements,
        api::achievements::get_achievement,
        api::achievements::put_achievement,
        api::achievements::get_completed,
        api::achievements::put_complete,
        api::achievements::delete_complete
    ),
    components(schemas(
        api::scores::Region,
        api::scores::ScoreAchievement,
        api::scores::ScoresAchievement,
        api::scores::damage::Character,
        api::scores::damage::ScoreDamage,
        api::scores::damage::ScoresDamage,
        api::scores::damage::DamageUpdate,
        api::scores::heal::ScoreHeal,
        api::scores::heal::ScoresHeal,
        api::scores::heal::HealUpdate,
        api::scores::shield::ScoreShield,
        api::scores::shield::ScoresShield,
        api::scores::shield::ShieldUpdate,
        api::users::User,
        api::users::UserLogin,
        api::users::UserRegister,
        api::users::EmailUpdate,
        api::users::PasswordUpdate,
        api::users::RequestToken,
        api::achievements::Diffuculty,
        api::achievements::Achievement,
        api::achievements::AchievementUpdate
    ))
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;

    env_logger::init();

    let _ = fs::create_dir("mihomo");

    let pool = PgPool::connect(&(dotenv::var("DATABASE_URL")?)).await?;

    sqlx::migrate!().run(&pool).await?;

    update::achievements(pool.clone()).await;
    update::scores().await;

    let password_resets = Data::new(Mutex::new(HashMap::<Uuid, String>::new()));
    let jwt_secret = Data::new(rand::thread_rng().gen::<[u8; 32]>());
    let pool = Data::new(pool);

    HttpServer::new(move || {
        App::new()
            .app_data(password_resets.clone())
            .app_data(jwt_secret.clone())
            .app_data(pool.clone())
            .service(Files::new("/static", "static").show_files_listing())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
            .wrap(Cors::permissive())
            .service(api::scores::damage::get_scores_damage)
            .service(api::scores::damage::get_score_damage)
            .service(api::scores::damage::put_score_damage)
            .service(api::scores::heal::get_scores_heal)
            .service(api::scores::heal::get_score_heal)
            .service(api::scores::heal::put_score_heal)
            .service(api::scores::shield::get_scores_shield)
            .service(api::scores::shield::get_score_shield)
            .service(api::scores::shield::put_score_shield)
            .service(api::scores::get_scores_achievement)
            .service(api::scores::get_score_achievement)
            .service(api::scores::put_score_achievement)
            .service(api::mihomo::get_mihomo)
            .service(api::users::register)
            .service(api::users::login)
            .service(api::users::logout)
            .service(api::users::request_token)
            .service(api::users::get_me)
            .service(api::users::put_email)
            .service(api::users::put_password)
            .service(api::achievements::get_completed)
            .service(api::achievements::put_complete)
            .service(api::achievements::delete_complete)
            .service(api::achievements::get_achievements)
            .service(api::achievements::get_achievement)
            .service(api::achievements::put_achievement)
        // .service(profile::get_profile)
    })
    .bind(("localhost", 8000))?
    .run()
    .await?;

    Ok(())
}
