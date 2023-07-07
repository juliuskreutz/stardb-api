mod api;
mod database;
mod mihomo;
mod update;

use api::*;
use rand::Rng;
use std::{collections::HashMap, sync::Mutex};

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
        scores::get_scores_achievement,
        scores::get_score_achievement,
        scores::put_score_achievement,
        scores::damage::get_scores_damage,
        scores::damage::get_score_damage,
        scores::damage::put_score_damage,
        scores::heal::get_scores_heal,
        scores::heal::get_score_heal,
        scores::heal::put_score_heal,
        scores::shield::get_scores_shield,
        scores::shield::get_score_shield,
        scores::shield::put_score_shield,
        users::login,
        users::register,
        users::logout,
        users::put_email,
        users::put_password,
        users::request_token,
        achievements::get_achievements,
        achievements::get_achievement,
        achievements::put_achievement,
        achievements::get_completed,
        achievements::put_complete,
        achievements::delete_complete
    ),
    components(schemas(
        scores::Region,
        scores::ScoreAchievement,
        scores::ScoreAchievementPartial,
        scores::ScoresAchievement,
        scores::damage::Character,
        scores::damage::ScoreDamage,
        scores::damage::ScoreDamagePartial,
        scores::damage::ScoresDamage,
        scores::damage::DamageUpdate,
        scores::heal::ScoreHeal,
        scores::heal::ScoreHealPartial,
        scores::heal::ScoresHeal,
        scores::heal::HealUpdate,
        scores::shield::ScoreShield,
        scores::shield::ScoreShieldPartial,
        scores::shield::ScoresShield,
        scores::shield::ShieldUpdate,
        users::User,
        users::UserLogin,
        users::UserRegister,
        users::EmailUpdate,
        users::PasswordUpdate,
        users::RequestToken,
        achievements::Diffuculty,
        achievements::Achievement,
        achievements::AchievementUpdate
    ))
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv::dotenv()?;

    env_logger::init();

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
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
            .wrap(Cors::permissive())
            .service(scores::damage::get_scores_damage)
            .service(scores::damage::get_score_damage)
            .service(scores::damage::put_score_damage)
            .service(scores::heal::get_scores_heal)
            .service(scores::heal::get_score_heal)
            .service(scores::heal::put_score_heal)
            .service(scores::shield::get_scores_shield)
            .service(scores::shield::get_score_shield)
            .service(scores::shield::put_score_shield)
            .service(scores::get_scores_achievement)
            .service(scores::get_score_achievement)
            .service(scores::put_score_achievement)
            .service(users::register)
            .service(users::login)
            .service(users::logout)
            .service(users::put_email)
            .service(users::put_password)
            .service(users::request_token)
            .service(achievements::get_completed)
            .service(achievements::put_complete)
            .service(achievements::delete_complete)
            .service(achievements::get_achievements)
            .service(achievements::get_achievement)
            .service(achievements::put_achievement)
        // .service(profile::get_profile)
    })
    .bind(("localhost", 8000))?
    .run()
    .await?;

    Ok(())
}
