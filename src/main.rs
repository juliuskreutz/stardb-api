mod api;
mod database;
mod mihomo;
mod update;

use std::{collections::HashMap, fs, sync::Mutex};

use actix_cors::Cors;
use actix_files::Files;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, web::Data, App, HttpServer};
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[derive(OpenApi)]
#[openapi(
    tags((name = "pinned")),
    paths(
        api::mihomo::get_mihomo,
        api::users::login,
        api::users::register,
        api::users::logout,
        api::users::request_token,
        api::users::get_me,
        api::users::put_email,
        api::users::put_password,
        api::users::delete_email,
        api::users::get_verifications,
        api::users::put_verification,
        api::users::get_user_achievements,
        api::users::put_user_achievement,
        api::users::delete_user_achievement,
        api::submissions::damage::get_submissions_damage,
        api::submissions::damage::get_submission_damage,
        api::submissions::damage::post_submission_damage,
        api::submissions::damage::delete_submission_damage,
        api::submissions::heal::get_submissions_heal,
        api::submissions::heal::get_submission_heal,
        api::submissions::heal::post_submission_heal,
        api::submissions::heal::delete_submission_heal,
        api::submissions::shield::get_submissions_shield,
        api::submissions::shield::get_submission_shield,
        api::submissions::shield::post_submission_shield,
        api::submissions::shield::delete_submission_shield,
        api::import::import,
    ),
    components(schemas(
        api::schemas::Region,
        api::schemas::ScoreAchievement,
        api::schemas::ScoresAchievement,
        api::schemas::ScoreDamage,
        api::schemas::ScoresDamage,
        api::schemas::ScoreHeal,
        api::schemas::ScoresHeal,
        api::schemas::ScoreShield,
        api::schemas::ScoresShield,
        api::schemas::SubmissionDamage,
        api::schemas::SubmissionDamageUpdate,
        api::schemas::SubmissionHeal,
        api::schemas::SubmissionHealUpdate,
        api::schemas::SubmissionShield,
        api::schemas::SubmissionShieldUpdate,
        api::schemas::DamageUpdate,
        api::schemas::HealUpdate,
        api::schemas::ShieldUpdate,
        api::users::User,
        api::users::UserLogin,
        api::users::UserRegister,
        api::users::EmailUpdate,
        api::users::PasswordUpdate,
        api::users::RequestToken,
        api::users::Verification,
        api::users::Otp,
        api::import::File
    ))
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init();

    let _ = fs::create_dir("mihomo");

    let pool = PgPool::connect(dotenv_codegen::dotenv!("DATABASE_URL")).await?;

    sqlx::migrate!().run(&pool).await?;

    update::achievements(pool.clone()).await;
    update::verifications(pool.clone()).await;
    update::scores().await;

    let password_resets = Data::new(Mutex::new(HashMap::<Uuid, String>::new()));
    let pool = Data::new(pool);

    let key = Key::generate();

    let mut openapi = ApiDoc::openapi();
    openapi.merge(api::openapi());

    HttpServer::new(move || {
        App::new()
            .app_data(password_resets.clone())
            .app_data(pool.clone())
            .wrap(Cors::permissive())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                key.clone(),
            ))
            .service(Files::new("/static", "static").show_files_listing())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi.clone()),
            )
            .configure(api::configure)
            .service(api::mihomo::get_mihomo)
            .service(api::users::register)
            .service(api::users::login)
            .service(api::users::logout)
            .service(api::users::request_token)
            .service(api::users::get_me)
            .service(api::users::put_email)
            .service(api::users::put_password)
            .service(api::users::delete_email)
            .service(api::users::get_verifications)
            .service(api::users::put_verification)
            .service(api::users::get_user_achievements)
            .service(api::users::put_user_achievement)
            .service(api::users::delete_user_achievement)
            .service(api::submissions::damage::get_submissions_damage)
            .service(api::submissions::damage::get_submission_damage)
            .service(api::submissions::damage::post_submission_damage)
            .service(api::submissions::damage::delete_submission_damage)
            .service(api::submissions::heal::get_submissions_heal)
            .service(api::submissions::heal::get_submission_heal)
            .service(api::submissions::heal::post_submission_heal)
            .service(api::submissions::heal::delete_submission_heal)
            .service(api::submissions::shield::get_submissions_shield)
            .service(api::submissions::shield::get_submission_shield)
            .service(api::submissions::shield::post_submission_shield)
            .service(api::submissions::shield::delete_submission_shield)
            .service(api::import::import)
    })
    .bind(("localhost", 8000))?
    .run()
    .await?;

    Ok(())
}
