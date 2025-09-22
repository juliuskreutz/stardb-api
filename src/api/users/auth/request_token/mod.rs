use std::{collections::HashMap, env};

use actix_web::{post, rt, web, HttpResponse, Responder};
use futures::lock::Mutex;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{api::ApiResult, database};

type TokenMap = Mutex<HashMap<Uuid, String>>;

lazy_static::lazy_static! {
    static ref CACHE: std::sync::Mutex<Option<web::Data<TokenMap>>> = std::sync::Mutex::new(None);
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/auth/request-token")),
    paths(request_token),
    components(schemas(RequestToken))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    let data = CACHE
        .lock()
        .unwrap()
        .get_or_insert_with(web::Data::default)
        .clone();

    cfg.service(request_token).app_data(data);
}

#[derive(Deserialize, ToSchema)]
pub struct RequestToken {
    email: String,
}

#[utoipa::path(
    tag = "users/auth/request-token",
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
    request_token: web::Json<RequestToken>,
    tokens: web::Data<Mutex<HashMap<Uuid, String>>>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let email = request_token.email.trim().to_string();

    let users = database::users::get_by_email(&email, &pool).await?;

    if users.is_empty() {
        return Ok(HttpResponse::BadRequest().finish());
    };

    for user in users {
        let username = user.username.clone();

        if tokens.lock().await.values().any(|s| s == &username) {
            continue;
        }

        let to = format!("{username} <{email}>").parse()?;

        let token = Uuid::new_v4();
        let email = Message::builder()
            .from("stardb <noreply@stardb.gg>".parse()?)
            .to(to)
            .subject("StarDB.GG Emergency Login")
            .body(format!(
                "Use the following link to login:\nhttps://stardb.gg/login?token={token}"
            ))?;

        let credentials = Credentials::new(env::var("SMTP_USERNAME")?, env::var("SMTP_PASSWORD")?);

        let mailer = SmtpTransport::relay("smtppro.zoho.eu")?
            .credentials(credentials)
            .build();

        mailer.send(&email)?;

        tokens.lock().await.insert(token, username.clone());

        let tokens = tokens.clone();
        rt::spawn(async move {
            rt::time::sleep(std::time::Duration::from_secs(5 * 60)).await;

            tokens.lock().await.remove(&token);
        });
    }

    Ok(HttpResponse::Ok().finish())
}
