use std::{collections::HashMap, sync::Mutex};

use actix_web::{post, rt, web, HttpResponse, Responder};
use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::{database, Result};

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
