mod uid;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, OpenApi};

use crate::{
    api::{private, scores::Region, ApiResult},
    database,
};

#[derive(OpenApi)]
#[openapi(paths(get_leaderboard))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(uid::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_leaderboard).configure(uid::configure);
}

#[derive(Serialize)]
struct Leaderboard {
    count: i64,
    count_na: i64,
    count_eu: i64,
    count_asia: i64,
    count_cn: i64,
    count_query: i64,
    scores: Vec<Score>,
}

#[derive(Serialize)]
struct Score {
    global_rank: i64,
    regional_rank: i64,
    uid: i64,
    region: Region,
    name: String,
    level: i32,
    signature: String,
    avatar_icon: String,
    achievement_count: i32,
    updated_at: NaiveDateTime,
}

#[derive(Deserialize, IntoParams)]
struct LeaderboardParams {
    region: Option<Region>,
    query: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl From<database::DbScoreAchievement> for Score {
    fn from(db_score: database::DbScoreAchievement) -> Self {
        Score {
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

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/leaderboard",
    params(LeaderboardParams),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "Leaderboard"),
    )
)]
#[get("/api/pages/leaderboard", guard = "private")]
async fn get_leaderboard(
    leaderboard_params: web::Query<LeaderboardParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let count_na =
        database::count_scores_achievement(Some(&Region::Na.to_string()), None, &pool).await?;
    let count_eu =
        database::count_scores_achievement(Some(&Region::Eu.to_string()), None, &pool).await?;
    let count_asia =
        database::count_scores_achievement(Some(&Region::Asia.to_string()), None, &pool).await?;
    let count_cn =
        database::count_scores_achievement(Some(&Region::Cn.to_string()), None, &pool).await?;
    let count_query = database::count_scores_achievement(
        leaderboard_params.region.map(|r| r.to_string()).as_deref(),
        leaderboard_params.query.as_deref(),
        &pool,
    )
    .await?;

    let count = count_na + count_eu + count_asia + count_cn;

    let db_scores = database::get_scores_achievement(
        leaderboard_params.region.map(|r| r.to_string()).as_deref(),
        leaderboard_params.query.as_deref(),
        leaderboard_params.limit,
        leaderboard_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores.into_iter().map(Score::from).collect();

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
