use std::io::BufReader;

use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_session::Session;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/import"), (name = "users/me/import-file")),
    paths(import, import_file),
    components(schemas(
        ImportData,
        File,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(import).service(import_file);
}

#[derive(Deserialize, ToSchema)]
struct ImportData {
    achievements: Option<Vec<i32>>,
    books: Option<Vec<i32>>,
}

#[derive(MultipartForm, ToSchema)]
struct File {
    #[schema(value_type = String, format = Binary)]
    file: TempFile,
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
    import_data: web::Json<ImportData>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if let Some(achievements) = &import_data.achievements {
        database::delete_user_achievements_completed(&username, &pool).await?;
        let mut achievement_completed = database::DbUserAchievementCompleted {
            username: username.clone(),
            id: 0,
        };
        for &achievement in achievements {
            achievement_completed.id = achievement;

            database::add_user_achievement_completed(&achievement_completed, &pool).await?;
        }
    }

    if let Some(books) = &import_data.books {
        database::delete_user_books_completed(&username, &pool).await?;
        let mut book_completed = database::DbUserBookCompleted {
            username: username.clone(),
            id: 0,
        };
        for &book in books {
            book_completed.id = book;

            database::add_user_book_completed(&book_completed, &pool).await?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/import-file",
    put,
    path = "/api/users/me/import-file",
    request_body(content = File, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/import-file")]
async fn import_file(
    session: Session,
    file: MultipartForm<File>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let import_data: ImportData = serde_json::from_reader(BufReader::new(&file.file.file))?;

    if let Some(achievements) = &import_data.achievements {
        database::delete_user_achievements_completed(&username, &pool).await?;
        let mut achievement_completed = database::DbUserAchievementCompleted {
            username: username.clone(),
            id: 0,
        };
        for &achievement in achievements {
            achievement_completed.id = achievement;

            database::add_user_achievement_completed(&achievement_completed, &pool).await?;
        }
    }

    if let Some(books) = &import_data.books {
        database::delete_user_books_completed(&username, &pool).await?;
        let mut book_completed = database::DbUserBookCompleted {
            username: username.clone(),
            id: 0,
        };
        for &book in books {
            book_completed.id = book;

            database::add_user_book_completed(&book_completed, &pool).await?;
        }
    }
    Ok(HttpResponse::Ok().finish())
}
