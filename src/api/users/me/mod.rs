mod achievements;
mod books;
mod email;
mod password;
mod uids;
mod username;

use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me")),
    paths(get_me),
    components(schemas(
        User,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievements::openapi());
    openapi.merge(books::openapi());
    openapi.merge(email::openapi());
    openapi.merge(password::openapi());
    openapi.merge(uids::openapi());
    openapi.merge(username::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_me)
        .configure(achievements::configure)
        .configure(books::configure)
        .configure(email::configure)
        .configure(password::configure)
        .configure(uids::configure)
        .configure(username::configure);
}

#[derive(Serialize, ToSchema)]
pub struct User {
    username: String,
    admin: bool,
    email: Option<String>,
    uids: Vec<i64>,
    achievements: Vec<i64>,
    books: Vec<i64>,
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
async fn get_me(session: Session, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    session.renew();

    let admin = database::get_admin_by_username(&username, &pool)
        .await
        .is_ok();

    let user = database::get_user_by_username(&username, &pool).await?;

    let email = user.email;

    let uids = database::get_connections_by_username(&username, &pool)
        .await?
        .into_iter()
        .map(|c| c.uid)
        .collect();

    let books = database::get_user_books_by_username(&username, &pool)
        .await?
        .into_iter()
        .map(|b| b.id)
        .collect();

    let achievements = database::get_user_achievements_by_username(&username, &pool)
        .await?
        .into_iter()
        .map(|b| b.id)
        .collect();

    let user = User {
        username,
        admin,
        email,
        uids,
        achievements,
        books,
    };

    Ok(HttpResponse::Ok().json(user))
}
