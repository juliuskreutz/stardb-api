mod verifications;

use actix_session::Session;
use actix_web::{delete, get, put, web, HttpResponse, Responder};
use argon2::Config;
use rand::Rng;
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
    paths(get_me, put_email, delete_email, put_password, get_user_achievements, put_user_achievements, put_user_achievement, delete_user_achievement),
    components(schemas(
        User,
        Verification,
        EmailUpdate,
        PasswordUpdate
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(verifications::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_me)
        .service(put_email)
        .service(delete_email)
        .service(put_password)
        .service(get_user_achievements)
        .service(put_user_achievements)
        .service(put_user_achievement)
        .service(delete_user_achievement)
        .configure(verifications::configure);
}

#[derive(Serialize, ToSchema)]
pub struct User {
    username: String,
    admin: bool,
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
async fn get_me(session: Session) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let user = User { username, admin };

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
            otp: db_verification.token,
        }
    }
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
    path = "/api/users/me/achievements",
    request_body = Vec<i64>,
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/achievements")]
async fn put_user_achievements(
    session: Session,
    ids: web::Json<Vec<i64>>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let mut complete = DbComplete { username, id: 0 };

    for id in ids.0 {
        complete.id = id;

        database::add_complete(&complete, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
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
