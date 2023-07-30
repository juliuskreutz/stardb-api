use actix_session::Session;
use actix_web::{delete, get, put, web, HttpResponse, Responder};
use argon2::Config;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::{self, DbComplete, DbVerification},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me")),
    paths(get_me, get_verifications, put_verification, put_email, delete_email, put_password),
    components(schemas(
        User,
        Verification,
        Otp,
        EmailUpdate,
        PasswordUpdate
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_me)
        .service(get_verifications)
        .service(put_verification)
        .service(put_email)
        .service(delete_email)
        .service(put_password);
}

#[derive(Serialize, ToSchema)]
pub struct User {
    username: String,
    email: Option<String>,
    admin: bool,
    uids: Vec<i64>,
    achievements: Vec<i64>,
}

#[utoipa::path(
    tag = "users/me",
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "User", body = User),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me")]
async fn get_me(session: Session, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let user = database::get_user_by_username(&username, &pool).await?;

    let username = user.username.clone();
    let email = user.email.clone();
    let admin = user.admin;

    let uids = database::get_connections_by_username(&username, &pool)
        .await?
        .into_iter()
        .map(|c| c.uid)
        .collect();

    let achievements = database::get_completed_by_username(&username, &pool)
        .await?
        .into_iter()
        .map(|c| c.id)
        .collect();

    let user = User {
        username,
        email,
        admin,
        uids,
        achievements,
    };

    Ok(HttpResponse::Ok().json(user))
}

#[derive(Serialize, ToSchema)]
pub struct Verification {
    uid: i64,
    otp: String,
}

impl From<DbVerification> for Verification {
    fn from(db_verification: DbVerification) -> Self {
        Verification {
            uid: db_verification.uid,
            otp: db_verification.otp,
        }
    }
}

#[utoipa::path(
    tag = "users/me",
    get,
    path = "/api/users/me/verifications",
    responses(
        (status = 200, description = "Verifications", body = Vec<Verification>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/verifications")]
async fn get_verifications(session: Session, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let db_verifications = database::get_verifications_by_username(&username, &pool).await?;

    let verifications: Vec<_> = db_verifications
        .into_iter()
        .map(Verification::from)
        .collect();

    Ok(HttpResponse::Ok().json(verifications))
}

#[derive(Serialize, ToSchema)]
pub struct Otp {
    otp: String,
}

#[utoipa::path(
    tag = "users/me",
    put,
    path = "/api/users/me/verifications/{uid}",
    responses(
        (status = 200, description = "Added verification", body = Otp),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/verifications/{uid}")]
async fn put_verification(
    session: Session,
    uid: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let otp: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();

    let db_verification = DbVerification {
        uid: *uid,
        username,
        otp,
    };

    database::set_verification(&db_verification, &pool).await?;

    let otp = Otp {
        otp: db_verification.otp.clone(),
    };

    Ok(HttpResponse::Ok().json(otp))
}

#[derive(Deserialize, ToSchema)]
pub struct EmailUpdate {
    email: String,
}

#[utoipa::path(
    tag = "users/me",
    put,
    path = "/api/users/me/email",
    request_body = EmailUpdate,
    responses(
        (status = 200, description = "Updated email"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/email")]
async fn put_email(
    session: Session,
    email_update: web::Json<EmailUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    database::update_user_email(&username, &email_update.email, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me",
    delete,
    path = "/api/users/me/email",
    responses(
        (status = 200, description = "Deleted email"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/email")]
async fn delete_email(session: Session, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    database::delete_user_email(&username, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, ToSchema)]
pub struct PasswordUpdate {
    password: String,
}

#[utoipa::path(
    tag = "users/me",
    put,
    path = "/api/users/me/password",
    request_body = PasswordUpdate,
    responses(
        (status = 200, description = "Updated password"),
    )
)]
#[put("/api/users/me/password")]
async fn put_password(
    session: Session,
    password_update: web::Json<PasswordUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let salt = rand::thread_rng().gen::<[u8; 32]>();

    let password = argon2::hash_encoded(
        password_update.password.as_bytes(),
        &salt,
        &Config::default(),
    )?;

    database::update_user_password(&username, &password, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me",
    get,
    path = "/api/users/me/achievements",
    responses(
        (status = 200, description = "Achievement ids", body = Vec<i64>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/achievements")]
async fn get_user_achievements(
    session: Session,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let completed: Vec<_> = database::get_completed_by_username(&username, &pool)
        .await?
        .iter()
        .map(|c| c.id)
        .collect();

    Ok(HttpResponse::Ok().json(completed))
}

#[utoipa::path(
    tag = "users/me",
    put,
    path = "/api/users/me/achievements/{id}",
    responses(
        (status = 200, description = "Successful add of the achievement"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/achievements/{id}")]
async fn put_user_achievement(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let db_complete = DbComplete { username, id };

    database::add_complete(&db_complete, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me",
    delete,
    path = "/api/users/me/achievements/{id}",
    responses(
        (status = 200, description = "Successful delete of the achievement"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/achievements/{id}")]
async fn delete_user_achievement(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let db_complete = DbComplete { username, id };

    database::delete_complete(&db_complete, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
