//! Transactional replacement of the signed-in user's supplied achievement lists across games.

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
        ImportData,
    ))
)]
struct ApiDoc;

/// Return the OpenAPI description for these routes, including registered child routes.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Register this module's HTTP routes and child route configuration.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(import);
}

#[derive(Deserialize, ToSchema)]
struct ImportData {
    hsr_achievements: Option<Vec<i32>>,
    gi_achievements: Option<Vec<i32>>,
    zzz_achievements: Option<Vec<i32>>,
}

#[utoipa::path(
    tag = "users/me/import",
    put,
    path = "/api/users/me/import",
    request_body = ImportData,
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in or unknown achievement id"),
    )
)]
#[put("/api/users/me/import")]
/// Replace supplied HSR/GI/ZZZ completion lists in one transaction for the signed-in user.
/// Omitted games remain untouched; an unknown ID in any game rolls back the complete import.
async fn import(
    session: Session,
    import_data: web::Json<ImportData>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    use database::achievement_lists::{apply, Game, List, UnknownAchievementId};
    let mut tx = pool.begin().await?;
    for (game, ids) in [
        (Game::Hsr, &import_data.hsr_achievements),
        (Game::Gi, &import_data.gi_achievements),
        (Game::Zzz, &import_data.zzz_achievements),
    ] {
        let Some(ids) = ids else {
            continue;
        };
        if let Err(error) = apply(game, List::Completed, &username, ids, true, &mut tx).await {
            if let Some(UnknownAchievementId(id)) = error.downcast_ref() {
                return Ok(HttpResponse::BadRequest().body(format!("Unknown achievement id {id}")));
            }
            return Err(error.into());
        }
    }
    tx.commit().await?;

    Ok(HttpResponse::Ok().finish())
}
