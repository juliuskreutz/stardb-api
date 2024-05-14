use actix_session::Session;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/import")),
    paths(import),
    components(schemas(
        Import,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(import);
}

#[derive(Deserialize, ToSchema)]
struct Import {
    achievements: Vec<i32>,
    books: Vec<i32>,
}

#[utoipa::path(
    tag = "users/me/import",
    put,
    path = "/api/users/me/import",
    request_body = Import,
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/import")]
async fn import(
    session: Session,
    import: web::Json<Import>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    database::delete_user_achievements_completed(&username, &pool).await?;
    let mut achievement_completed = database::DbUserAchievementCompleted {
        username: username.clone(),
        id: 0,
    };
    for &achievement in &import.achievements {
        achievement_completed.id = achievement;

        database::add_user_achievement_completed(&achievement_completed, &pool).await?;
    }

    database::delete_user_books_completed(&username, &pool).await?;
    let mut book_completed = database::DbUserBookCompleted {
        username: username.clone(),
        id: 0,
    };
    for &book in &import.books {
        book_completed.id = book;

        database::add_user_book_completed(&book_completed, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
