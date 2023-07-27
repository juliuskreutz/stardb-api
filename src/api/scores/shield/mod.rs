mod uid;

use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{schemas::*, scores::ScoresParams},
    database::{self, DbScoreShield},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores/shield")),
    paths(get_scores_shield),
    components(schemas(
        ScoreShield
    ))
)]
struct ApiDoc;

impl From<DbScoreShield> for ScoreShield {
    fn from(db_score: DbScoreShield) -> Self {
        ScoreShield {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            shield: db_score.shield,
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
    cfg.service(get_scores_shield).configure(uid::configure);
}

#[utoipa::path(
    tag = "scores/shield",
    get,
    path = "/api/scores/shield",
    params(
        ScoresParams
    ),
    responses(
        (status = 200, description = "ScoresShield", body = ScoresShield),
    )
)]
#[get("/api/scores/shield")]
async fn get_scores_shield(
    scores_params: web::Query<ScoresParams>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let count_na = database::count_scores_shield(&Region::NA.to_string(), &pool).await?;
    let count_eu = database::count_scores_shield(&Region::EU.to_string(), &pool).await?;
    let count_asia = database::count_scores_shield(&Region::Asia.to_string(), &pool).await?;
    let count_cn = database::count_scores_shield(&Region::CN.to_string(), &pool).await?;

    let count = count_na + count_eu + count_asia + count_cn;

    let db_scores_shield = database::get_scores_shield(
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores_shield
        .into_iter()
        .map(ScoreShield::from)
        .collect();

    let scores_shield = Scores {
        count,
        count_na,
        count_eu,
        count_asia,
        count_cn,
        scores,
    };

    Ok(HttpResponse::Ok().json(scores_shield))
}
