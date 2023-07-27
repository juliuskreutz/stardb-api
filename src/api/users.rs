use std::{collections::HashMap, sync::Mutex};

use actix_session::Session;
use actix_web::{
    delete, get, post, put,
    rt::{self, time},
    web, HttpResponse, Responder,
};
use argon2::Config;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database::{self, DbComplete, DbUser, DbVerification};
use crate::Result;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum UserLogin {
    UsernamePassword { username: String, password: String },
    Token { token: String },
}

#[utoipa::path(
    post,
    path = "/api/users/login",
    request_body(content = UserLogin,
        examples(
            ("UsernamePassword" = (value = json!({"username": "meow", "password": "meow12"}))),
            ("Token" = (value = json!({"token": "a3449001-8762-48e2-8813-3abe92a29884"})))
        )
    ),
    responses(
        (status = 200, description = "Successfull login. The session id is returned in a cookie named `id`. You need to include this cookie in subsequent requests"),
        (status = 400, description = "Don't have an account")
    )
)]
#[post("/api/users/login")]
async fn login(
    session: Session,
    user_login: web::Json<UserLogin>,
    password_resets: web::Data<Mutex<HashMap<Uuid, String>>>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let username = match &*user_login {
        UserLogin::UsernamePassword { username, password } => {
            let Ok(user) = database::get_user_by_username(username, &pool).await else {
                return Ok(HttpResponse::BadRequest().finish());
            };

            if !argon2::verify_encoded(&user.password, password.as_bytes()).unwrap_or_default() {
                return Ok(HttpResponse::BadRequest().finish());
            }

            username.clone()
        }
        UserLogin::Token { token } => {
            let Some(username) = password_resets
                .lock()
                .map_err(|_| "lock broken")?
                .remove(&token.parse()?)
            else {
                return Ok(HttpResponse::BadRequest().finish());
            };

            username.clone()
        }
    };

    let user = database::get_user_by_username(&username, &pool).await?;

    session.insert("username", user.username)?;
    session.insert("admin", user.admin)?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, ToSchema)]
pub struct UserRegister {
    username: String,
    password: String,
    email: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/users/register",
    request_body(content = UserRegister,
        examples(
            ("UsernamePassword" = (value = json!({"username": "meow", "password": "meow12"}))),
            ("UsernamePasswordEmail" = (value = json!({"username": "meow", "password": "meow12", "email": "meow@gmail.com"})))
        )
    ),
    responses(
        (status = 200, description = "Successfull register. The session id is returned in a cookie named `id`. You need to include this cookie in subsequent requests"),
        (status = 400, description = "Credentials too long"),
        (status = 409, description = "Account already exists")
    )
)]
#[post("/api/users/register")]
async fn register(
    session: Session,
    user_register: web::Json<UserRegister>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let username = user_register.username.clone();
    let password = user_register.password.clone();
    let email = user_register.email.clone();

    if username.len() > 32
        || password.len() > 64
        || email.as_ref().map(|s| s.len()).unwrap_or_default() > 64
    {
        return Ok(HttpResponse::BadRequest().finish());
    }

    if database::get_user_by_username(&username, &pool)
        .await
        .is_ok()
    {
        return Ok(HttpResponse::Conflict().finish());
    }

    let salt = rand::thread_rng().gen::<[u8; 32]>();

    let password = argon2::hash_encoded(password.as_bytes(), &salt, &Config::default())?;

    {
        let username = username.clone();
        let user = DbUser {
            username,
            password,
            email,
            admin: false,
        };
        database::set_user(&user, &pool).await?;
    }

    session.insert("username", username)?;
    session.insert("admin", false)?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    post,
    path = "/api/users/logout",
    responses(
        (status = 200, description = "Successfull logout. The session id is deleted"),
    )
)]
#[post("/api/users/logout")]
async fn logout(session: Session) -> impl Responder {
    session.purge();

    HttpResponse::Ok().finish()
}

#[derive(Deserialize, ToSchema)]
pub struct RequestToken {
    username: String,
}

#[utoipa::path(
    post,
    path = "/api/users/request-token",
    request_body = RequestToken,
    responses(
        (status = 200, description = "Send mail with emergency login"),
        (status = 400, description = "No email connected"),
    )
)]
#[post("/api/users/request-token")]
async fn request_token(
    password_reset: web::Json<RequestToken>,
    password_resets: web::Data<Mutex<HashMap<Uuid, String>>>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(user) = database::get_user_by_username(&password_reset.username, &pool).await else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let Some(email) = user.email else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let to = format!("<{email}>").parse()?;

    let token = Uuid::new_v4();
    let email = Message::builder()
        .from("Julius Kreutz <noreply@kreutz.dev>".parse()?)
        .to(to)
        .subject("Stardb Password Reset")
        .body(format!("https://stardb.gg/login?token={token}"))?;

    let credentials = Credentials::new(
        dotenv_codegen::dotenv!("SMTP_USERNAME").to_string(),
        dotenv_codegen::dotenv!("SMTP_PASSWORD").to_string(),
    );

    let mailer = SmtpTransport::relay("mail.hosting.de")?
        .credentials(credentials)
        .build();

    mailer.send(&email)?;

    password_resets
        .lock()
        .map_err(|_| "lock broken")?
        .insert(token, user.username.clone());

    rt::spawn(async move {
        time::sleep(std::time::Duration::from_secs(5 * 60)).await;

        password_resets
            .lock()
            .map_err(|_| "lock broken")?
            .remove(&token);

        Result::<()>::Ok(())
    });

    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize, ToSchema)]
pub struct User {
    username: String,
    email: Option<String>,
    admin: bool,
    uids: Vec<i64>,
}

#[utoipa::path(
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
        .iter()
        .map(|c| c.uid)
        .collect();

    let user = User {
        username,
        email,
        admin,
        uids,
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
    get,
    path = "/api/users/verifications",
    responses(
        (status = 200, description = "Verifications", body = Vec<Verification>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/verifications")]
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
    put,
    path = "/api/users/verifications/{uid}",
    responses(
        (status = 200, description = "Added verification", body = Otp),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/verifications/{uid}")]
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
    put,
    path = "/api/users/email",
    request_body = EmailUpdate,
    responses(
        (status = 200, description = "Updated email"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/email")]
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
    delete,
    path = "/api/users/email",
    responses(
        (status = 200, description = "Deleted email"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/email")]
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
    put,
    path = "/api/users/password",
    request_body = PasswordUpdate,
    responses(
        (status = 200, description = "Updated password"),
    )
)]
#[put("/api/users/password")]
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
    get,
    path = "/api/users/achievements",
    responses(
        (status = 200, description = "Achievement ids", body = Vec<i64>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/achievements")]
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
    put,
    path = "/api/users/achievements/{id}",
    responses(
        (status = 200, description = "Successful add of the achievement"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/achievements/{id}")]
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
    delete,
    path = "/api/users/achievements/{id}",
    responses(
        (status = 200, description = "Successful delete of the achievement"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/achievements/{id}")]
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
