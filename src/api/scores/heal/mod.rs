mod uid;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::scores::{Scores, ScoresParams},
    database::{self, DbScoreHeal},
    Result,
};

use super::Region;

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores/heal")),
    paths(get_scores_heal),
    components(schemas(
        ScoreHeal
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
pub struct ScoreHeal {
    pub global_rank: i64,
    pub regional_rank: i64,
    pub uid: i64,
    pub heal: i32,
    pub video: String,
    pub region: Region,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub updated_at: NaiveDateTime,
}

impl From<DbScoreHeal> for ScoreHeal {
    fn from(db_score: DbScoreHeal) -> Self {
        ScoreHeal {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            heal: db_score.heal,
            video: db_score.video,
            region: db_score.region.parse().unwrap(),
            name: db_score.name,
            level: db_score.level,
            signature: db_score.signature,
            avatar_icon: db_score.avatar_icon,
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
    cfg.service(get_scores_heal).configure(uid::configure);
}

#[utoipa::path(
    tag = "scores/heal",
    get,
    path = "/api/scores/heal",
    params(
        ScoresParams
    ),
    responses(
        (status = 200, description = "ScoresHeal", body = ScoresHeal),
    )
)]
#[get("/api/scores/heal")]
async fn get_scores_heal(
    scores_params: web::Query<ScoresParams>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let count_na = database::count_scores_heal(&Region::NA.to_string(), &pool).await?;
    let count_eu = database::count_scores_heal(&Region::EU.to_string(), &pool).await?;
    let count_asia = database::count_scores_heal(&Region::Asia.to_string(), &pool).await?;
    let count_cn = database::count_scores_heal(&Region::CN.to_string(), &pool).await?;

    let count = count_na + count_eu + count_asia + count_cn;

    let db_scores_heal = database::get_scores_heal(
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores_heal.into_iter().map(ScoreHeal::from).collect();

    let scores_heal = Scores {
        count,
        count_na,
        count_eu,
        count_asia,
        count_cn,
        scores,
    };

    Ok(HttpResponse::Ok().json(scores_heal))
}
