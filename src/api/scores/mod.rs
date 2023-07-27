mod damage;
mod heal;
mod shield;
mod uid;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{IntoParams, OpenApi};

use crate::{
    api::schemas::*,
    database::{self, DbScore},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores")),
    paths(get_scores_achievement),
    components(schemas(
        ScoreAchievement
    ))
)]
struct ApiDoc;

#[derive(Deserialize, IntoParams)]
pub struct ScoresParams {
    pub region: Option<Region>,
    pub query: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
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
    openapi.merge(damage::openapi());
    openapi.merge(heal::openapi());
    openapi.merge(shield::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_scores_achievement)
        .configure(damage::configure)
        .configure(heal::configure)
        .configure(shield::configure)
        .configure(uid::configure);
}

#[utoipa::path(
    tag = "scores",
    get,
    path = "/api/scores",
    params(
        ScoresParams
    ),
    responses(
        (status = 200, description = "ScoresAchievement", body = ScoresAchievement),
    )
)]
#[get("/api/scores")]
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
