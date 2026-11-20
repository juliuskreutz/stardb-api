mod uid;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{scores::ScoresParams, ApiResult, Region},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores/achievements")),
    paths(get_scores_achievements),
    components(schemas(
        ScoreAchievement
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct ScoreAchievement {
    global_rank: i64,
    regional_rank: i64,
    uid: i32,
    region: Region,
    name: String,
    level: i32,
    signature: String,
    avatar_icon: String,
    achievement_count: i32,
    updated_at: DateTime<Utc>,
}

impl From<database::achievement_scores::DbScoreAchievement> for ScoreAchievement {
    /// Convert nonnullable database ranks and profile metadata to the public score payload.
    fn from(db_score: database::achievement_scores::DbScoreAchievement) -> Self {
        ScoreAchievement {
            global_rank: db_score.global_rank,
            regional_rank: db_score.regional_rank,
            uid: db_score.uid,
            region: db_score.region.parse().unwrap(),
            name: db_score.name,
            level: db_score.level,
            signature: db_score.signature,
            avatar_icon: db_score.avatar_icon,
            achievement_count: db_score.achievement_count,
            updated_at: db_score.updated_at,
        }
    }
}

/// Return the OpenAPI description for these routes, including registered child routes.
pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(uid::openapi());
    openapi
}

/// Register this module's HTTP routes and child route configuration.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_scores_achievements)
        .configure(uid::configure);
}

/// Rows returned when the caller gives no limit.
const DEFAULT_LIMIT: i64 = 50;
/// Upper bound per request; without it one call could serialize the whole leaderboard.
const MAX_LIMIT: i64 = 1_000;

/// Clamp the requested page size to 1..=MAX_LIMIT, defaulting to DEFAULT_LIMIT.
fn page_limit(limit: Option<i64>) -> i64 {
    limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

#[utoipa::path(
    tag = "scores/achievements",
    get,
    path = "/api/scores/achievements",
    params(
        ScoresParams
    ),
    responses(
        (status = 200, description = "[ScoreAchievement]", body = Vec<ScoreAchievement>),
    )
)]
#[get("/api/scores/achievements")]
/// Return ranked achievement scores with the requested region/name filters and pagination.
async fn get_scores_achievements(
    scores_params: web::Query<ScoresParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_scores = database::achievement_scores::get(
        scores_params.region.map(|r| r.to_string()).as_deref(),
        scores_params.query.as_deref(),
        Some(page_limit(scores_params.limit)),
        Some(scores_params.offset.unwrap_or(0).max(0)),
        &pool,
    )
    .await?;

    let scores: Vec<_> = db_scores.into_iter().map(ScoreAchievement::from).collect();

    Ok(HttpResponse::Ok().json(scores))
}

#[cfg(test)]
mod page_limit_tests {
    #[test]
    fn limits_default_and_clamp() {
        assert_eq!(super::page_limit(None), 50);
        assert_eq!(super::page_limit(Some(20)), 20);
        assert_eq!(super::page_limit(Some(0)), 1);
        assert_eq!(super::page_limit(Some(-5)), 1);
        assert_eq!(super::page_limit(Some(1_000_000)), 1_000);
    }
}
