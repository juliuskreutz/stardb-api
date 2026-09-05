use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use super::Leaderboard;
use crate::{
    api::{private, ApiResult},
    database, mihomo,
};

#[derive(OpenApi)]
#[openapi(paths(get_leaderboard_entry))]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module's routes and any shared application data.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_leaderboard_entry);
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/leaderboard/{uid}",
    security(("api_key" = [])),
    responses(
        (status = 200, description = "Leaderboard"),
    )
)]
#[get("/api/pages/leaderboard/{uid}", guard = "private")]
/// Returns one UID's score with regional totals after invoking mihomo profile seeding.
/// An invalid UID returns 400 and an absent score returns 404; profile fetches may
/// update the cache and database before the leaderboard is read.
async fn get_leaderboard_entry(
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    if !(100000000..1000000000).contains(&uid) {
        return Ok(HttpResponse::BadRequest().finish());
    }

    mihomo::ensure_row(uid, &pool).await?;

    let Some(score) = database::achievement_scores::get_by_uid(uid, &pool).await? else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let score = score.into();

    let counts = database::achievement_scores::leaderboard_counts(None, None, &pool).await?;
    let count = counts.total();
    let database::achievement_scores::LeaderboardCounts {
        count_na,
        count_eu,
        count_asia,
        count_cn,
        ..
    } = counts;
    let count_query = 1;

    let scores = vec![score];

    let leaderboard = Leaderboard {
        count,
        count_na,
        count_eu,
        count_asia,
        count_cn,
        count_query,
        scores,
    };

    Ok(HttpResponse::Ok().json(leaderboard))
}
