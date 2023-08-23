mod uid;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::{
    api::{scores::ScoresParams, ApiResult},
    database,
};

use super::Region;

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores/damage")),
    paths(get_scores_damage),
    components(schemas(
        ScoreDamage
    ))
)]
struct ApiDoc;

#[derive(Deserialize, IntoParams)]
pub struct DamageParams {
    pub character: Option<i32>,
    pub support: Option<bool>,
}

#[derive(Serialize, ToSchema)]
struct ScoreDamage {
    global_rank: i64,
    regional_rank: i64,
    uid: i64,
    character: i32,
    support: bool,
    damage: i32,
    video: String,
    region: Region,
    name: String,
    level: i32,
    signature: String,
    avatar_icon: String,
    updated_at: NaiveDateTime,
}

impl From<database::DbScoreDamage> for ScoreDamage {
    fn from(db_score: database::DbScoreDamage) -> Self {
        ScoreDamage {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            character: db_score.character,
            support: db_score.support,
            damage: db_score.damage,
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
    cfg.service(get_scores_damage).configure(uid::configure);
}

#[utoipa::path(
    tag = "scores/damage",
    get,
    path = "/api/scores/damage",
    params(
        DamageParams,
        ScoresParams
    ),
    responses(
        (status = 200, description = "[ScoreDamage]", body = Vec<ScoreDamage>),
    )
)]
#[get("/api/scores/damage")]
async fn get_scores_damage(
    damage_params: web::Query<DamageParams>,
    scores_params: web::Query<ScoresParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_scores_damage = database::get_scores_damage(
        damage_params.character,
        damage_params.support,
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores: Vec<_> = db_scores_damage
        .into_iter()
        .map(ScoreDamage::from)
        .collect();

    Ok(HttpResponse::Ok().json(scores))
}
