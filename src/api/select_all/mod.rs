use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "select-all")),
    paths(select_all),
    components(schemas(
        Username,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(select_all);
}

#[derive(Deserialize, ToSchema)]
struct Username {
    username: String,
}

#[utoipa::path(
    tag = "pinned",
    post,
    path = "/api/select-all",
    request_body(content = Username),
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in"),
        (status = 403, description = "Not an admin")
    )
)]
#[post("/api/select-all")]
async fn select_all(
    session: Session,
    username_json: web::Json<Username>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::admins::get_one_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::achievements::select_all(&username_json.username, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
