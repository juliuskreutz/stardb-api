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

impl From<database::DbScoreAchievement> for ScoreAchievement {
    fn from(db_score: database::DbScoreAchievement) -> Self {
        ScoreAchievement {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
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

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(uid::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_scores_achievements)
        .configure(uid::configure);
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
async fn get_scores_achievements(
    scores_params: web::Query<ScoresParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_scores = database::get_scores_achievement(
        scores_params.region.map(|r| r.to_string()).as_deref(),
        scores_params.query.as_deref(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores: Vec<_> = db_scores.into_iter().map(ScoreAchievement::from).collect();

    Ok(HttpResponse::Ok().json(scores))
}
