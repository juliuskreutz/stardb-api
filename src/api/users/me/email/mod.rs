use actix_session::Session;
use actix_web::{delete, get, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/email")),
    paths(get_email, put_email, delete_email),
    components(schemas(
        EmailUpdate
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_email)
        .service(put_email)
        .service(delete_email);
}

#[utoipa::path(
    tag = "users/me/email",
    get,
    path = "/api/users/me/email",
    responses(
        (status = 200, description = "Email", body = Option<String>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me/email")]
async fn get_email(session: Session, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let user = database::users::get_one_by_username(&username, &pool).await?;

    Ok(HttpResponse::Ok().json(user.email))
}

#[derive(Deserialize, ToSchema)]
pub struct EmailUpdate {
    email: String,
}

#[utoipa::path(
    tag = "users/me/email",
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
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    database::users::update_email_by_username(&username, &email_update.email, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/email",
    delete,
    path = "/api/users/me/email",
    responses(
        (status = 200, description = "Deleted email"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/email")]
async fn delete_email(session: Session, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    database::users::delete_email_by_username(&username, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
