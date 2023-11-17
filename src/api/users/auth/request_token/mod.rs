use std::{collections::HashMap, env};

use actix_web::{post, web, HttpResponse, Responder};
use futures::lock::Mutex;
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{api::ApiResult, database};

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
    cfg.service(request_token);
}

#[derive(Deserialize, ToSchema)]
pub struct RequestToken {
    username: String,
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
    let username = request_token.username.to_lowercase();

    {
        if tokens.lock().await.values().any(|s| s == &username) {
            return Ok(HttpResponse::BadRequest().finish());
        }
    }

    let Ok(user) = database::get_user_by_username(&username, &pool).await else {
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
        .subject("StarDB.GG Emergency Login")
        .body(format!("Click -> https://stardb.gg/login?token={token}"))?;

    let credentials = Credentials::new(env::var("SMTP_USERNAME")?, env::var("SMTP_PASSWORD")?);

    let mailer = SmtpTransport::relay("mail.hosting.de")?
        .credentials(credentials)
        .build();

    mailer.send(&email)?;

    tokens.lock().await.insert(token, username.clone());

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5 * 60)).await;

        tokens.lock().await.remove(&token);
    });

    Ok(HttpResponse::Ok().finish())
}
