use std::{collections::HashMap, sync::Mutex};

use actix_web::{
    cookie::{self, Cookie},
    post, put,
    rt::{self, time},
    web, HttpRequest, HttpResponse, Responder,
};
use argon2::Config;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::database;
use crate::Result;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub admin: bool,
    pub exp: usize,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct User {
    pub username: String,
    pub password: String,
    pub email: Option<String>,
}

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
        (status = 200, description = "Successfull login. The session token is returned in a cookie named `token`. You need to include this cookie in subsequent requests."),
        (status = 400, description = "Don't have an account.")
    )
)]
#[post("/api/users/login")]
async fn login(
    user_login: web::Json<UserLogin>,
    password_resets: web::Data<Mutex<HashMap<Uuid, String>>>,
    jwt_secret: web::Data<[u8; 32]>,
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

    let admin = database::is_admin(&username, &pool).await;
    let exp = (Utc::now() + Duration::hours(1)).timestamp() as usize;

    let claims = Claims {
        username,
        admin,
        exp,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(cookie::time::Duration::new(60 * 60, 0))
        .http_only(true)
        .secure(true)
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).finish())
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
        (status = 200, description = "Successfull register. The session token is returned in a cookie named `token`. You need to include this cookie in subsequent requests."),
        (status = 400, description = "Credentials too long."),
        (status = 409, description = "Account already exists.")
    )
)]
#[post("/api/users/register")]
async fn register(
    user: web::Json<UserRegister>,
    jwt_secret: web::Data<[u8; 32]>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let username = user.username.clone();
    let password = user.password.clone();
    let email = user.email.clone();

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
        let user = User {
            username,
            password,
            email,
        };
        database::set_user(&user, &pool).await?;
    }

    let admin = database::is_admin(&username, &pool).await;
    let exp = (Utc::now() + Duration::hours(1)).timestamp() as usize;

    let claims = Claims {
        username,
        admin,
        exp,
    };

    let token = jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .unwrap();

    let cookie = Cookie::build("token", token.to_owned())
        .path("/")
        .max_age(cookie::time::Duration::new(60 * 60, 0))
        .http_only(true)
        .secure(true)
        .finish();

    Ok(HttpResponse::Ok().cookie(cookie).finish())
}

#[utoipa::path(
    post,
    path = "/api/users/logout",
    responses(
        (status = 200, description = "Successfull logout. The session token is deleted."),
    )
)]
#[post("/api/users/logout")]
async fn logout(request: HttpRequest) -> impl Responder {
    let Some(mut cookie) = request.cookie("token") else {
        return HttpResponse::Ok().finish();
    };

    cookie.make_removal();

    HttpResponse::Ok().cookie(cookie).finish()
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
        (status = 200, description = "Updated email."),
        (status = 400, description = "Not logged in."),
    )
)]
#[put("/api/users/email")]
async fn put_email(
    request: HttpRequest,
    email_update: web::Json<EmailUpdate>,
    jwt_secret: web::Data<[u8; 32]>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Some(cookie) = request.cookie("token") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let claims: Claims = jsonwebtoken::decode(
        cookie.value(),
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|t| t.claims)?;

    database::update_password(&claims.username, &email_update.email, &pool).await?;

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
        (status = 200, description = "Updated password."),
    )
)]
#[put("/api/users/password")]
async fn put_password(
    request: HttpRequest,
    password_update: web::Json<PasswordUpdate>,
    jwt_secret: web::Data<[u8; 32]>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Some(cookie) = request.cookie("token") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let claims: Claims = jsonwebtoken::decode(
        cookie.value(),
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|t| t.claims)?;

    let salt = rand::thread_rng().gen::<[u8; 32]>();

    let password = argon2::hash_encoded(
        password_update.password.as_bytes(),
        &salt,
        &Config::default(),
    )?;

    database::update_password(&claims.username, &password, &pool).await?;

    Ok(HttpResponse::Ok().finish())
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
        (status = 200, description = "Send mail with emergency login."),
        (status = 400, description = "No email connected."),
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

    let token = Uuid::new_v4();
    password_resets
        .lock()
        .map_err(|_| "lock broken")?
        .insert(token, user.username.clone());

    let to = format!("<{email}>").parse()?;

    let email = Message::builder()
        .from("Julius Kreutz <noreply@kreutz.dev>".parse()?)
        .to(to)
        .subject("Stardb Password Reset")
        .body(format!("https://stardb.gg/login?token={token}"))?;

    let credentials =
        Credentials::new(dotenv::var("SMTP_USERNAME")?, dotenv::var("SMTP_PASSWORD")?);

    let mailer = SmtpTransport::relay("mail.hosting.de")?
        .credentials(credentials)
        .build();

    mailer.send(&email)?;

    rt::spawn(async move {
        let mut interval = time::interval(std::time::Duration::from_secs(5 * 60));

        interval.tick().await;
        interval.tick().await;

        password_resets
            .lock()
            .map_err(|_| "lock broken")?
            .remove(&token);

        Result::<()>::Ok(())
    });

    Ok(HttpResponse::Ok().finish())
}
