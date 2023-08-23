mod uid;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{scores::ScoresParams, ApiResult},
    database,
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
struct ScoreHeal {
    global_rank: i64,
    regional_rank: i64,
    uid: i64,
    heal: i32,
    video: String,
    region: Region,
    name: String,
    level: i32,
    signature: String,
    avatar_icon: String,
    updated_at: NaiveDateTime,
}

impl From<database::DbScoreHeal> for ScoreHeal {
    fn from(db_score: database::DbScoreHeal) -> Self {
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
        (status = 200, description = "[ScoreHeal]", body = Vec<ScoreHeal>),
    )
)]
#[get("/api/scores/heal")]
async fn get_scores_heal(
    scores_params: web::Query<ScoresParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_scores_heal = database::get_scores_heal(
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores: Vec<_> = db_scores_heal.into_iter().map(ScoreHeal::from).collect();

    Ok(HttpResponse::Ok().json(scores))
}
