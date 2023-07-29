mod uid;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::scores::{Region, Scores, ScoresParams},
    database::{self, DbScore},
    Result,
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

impl From<DbScore> for ScoreAchievement {
    fn from(db_score: DbScore) -> Self {
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
) -> Result<impl Responder> {
    let count_na = database::count_scores(&Region::NA.to_string(), &pool).await?;
    let count_eu = database::count_scores(&Region::EU.to_string(), &pool).await?;
    let count_asia = database::count_scores(&Region::Asia.to_string(), &pool).await?;
    let count_cn = database::count_scores(&Region::CN.to_string(), &pool).await?;

    let count = count_na + count_eu + count_asia + count_cn;

    let db_scores = database::get_scores(
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
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
        scores,
    };

    Ok(HttpResponse::Ok().json(scores_achievement))
}
