use actix_session::Session;
use actix_web::{put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/gi/import")),
    paths(gi_import),
    components(schemas(
        ImportData,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(gi_import);
}

#[derive(Deserialize, ToSchema)]
struct ImportData {
    achievements: Option<Vec<i32>>,
}

#[utoipa::path(
    tag = "users/me/gi/import",
    put,
    path = "/api/users/me/gi/import",
    request_body = ImportData,
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/gi/import")]
async fn gi_import(
    session: Session,
    import_data: web::Json<ImportData>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if let Some(achievements) = &import_data.achievements {
        database::gi::users_achievements_completed::delete_by_username(&username, &pool).await?;

        let mut achievement_completed =
            database::gi::users_achievements_completed::DbUserAchievementCompleted {
                username: username.clone(),
                id: 0,
            };
        for &achievement in achievements {
            achievement_completed.id = achievement;

            database::gi::users_achievements_completed::add(&achievement_completed, &pool).await?;
        }
    }

    Ok(HttpResponse::Ok().finish())
}
