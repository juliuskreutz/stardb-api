use std::{collections::HashMap, sync::Mutex};

use actix_session::Session;
use actix_web::{post, rt, web, HttpResponse, Responder};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{
    database::{self, DbUser},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/auth")),
    paths(login, register, logout, request_token),
    components(schemas(UserLogin, UserRegister, RequestToken))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(login)
        .service(register)
        .service(logout)
        .service(request_token);
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum UserLogin {
    UsernamePassword { username: String, password: String },
    Token { token: String },
}

#[utoipa::path(
    tag = "pinned",
    post,
    path = "/api/users/auth/login",
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
#[post("/api/users/auth/login")]
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
    tag = "users/auth",
    post,
    path = "/api/users/auth/register",
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
#[post("/api/users/auth/register")]
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

    let password = argon2::hash_encoded(password.as_bytes(), &salt, &argon2::Config::default())?;

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
    tag = "users/auth",
    post,
    path = "/api/users/auth/logout",
    responses(
        (status = 200, description = "Successfull logout. The session id is deleted"),
    )
)]
#[post("/api/users/auth/logout")]
async fn logout(session: Session) -> impl Responder {
    session.purge();

    HttpResponse::Ok().finish()
}

#[derive(Deserialize, ToSchema)]
pub struct RequestToken {
    username: String,
}

#[utoipa::path(
    tag = "users/auth",
    post,
    path = "/api/users/auth/request-token",
    request_body = RequestToken,
    responses(
        (status = 200, description = "Send mail with emergency login"),
        (status = 400, description = "No email connected"),
    )
)]
#[post("/api/users/auth/request-token")]
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
        rt::time::sleep(std::time::Duration::from_secs(5 * 60)).await;

        password_resets
            .lock()
            .map_err(|_| "lock broken")?
            .remove(&token);

        Result::<()>::Ok(())
    });

    Ok(HttpResponse::Ok().finish())
}
