mod uid;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{
        scores::{Region, Scores, ScoresParams},
        ApiResult,
    },
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores/achievements")),
    paths(get_scores_achievement),
    components(schemas(
        ScoreAchievement
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
pub struct ScoreAchievement {
    pub global_rank: i64,
    pub regional_rank: i64,
    pub uid: i64,
    pub region: Region,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub achievement_count: i32,
    pub updated_at: NaiveDateTime,
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
    cfg.service(get_scores_achievement)
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
        (status = 200, description = "ScoresAchievement", body = ScoresAchievement),
    )
)]
#[get("/api/scores/achievements")]
async fn get_scores_achievement(
    scores_params: web::Query<ScoresParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let count_na =
        database::count_scores_achievement(Some(&Region::NA.to_string()), None, &pool).await?;
    let count_eu =
        database::count_scores_achievement(Some(&Region::EU.to_string()), None, &pool).await?;
    let count_asia =
        database::count_scores_achievement(Some(&Region::Asia.to_string()), None, &pool).await?;
    let count_cn =
        database::count_scores_achievement(Some(&Region::CN.to_string()), None, &pool).await?;
    let count_query = database::count_scores_achievement(
        scores_params.region.map(|r| r.to_string()).as_deref(),
        scores_params.query.as_deref(),
        &pool,
    )
    .await?;

    let count = count_na + count_eu + count_asia + count_cn;

    let db_scores = database::get_scores_achievement(
        scores_params.region.map(|r| r.to_string()).as_deref(),
        scores_params.query.as_deref(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores.into_iter().map(ScoreAchievement::from).collect();

    let scores_achievement = Scores {
        count,
        count_na,
        count_eu,
        count_asia,
        count_cn,
        count_query,
        scores,
    };

    Ok(HttpResponse::Ok().json(scores_achievement))
}
