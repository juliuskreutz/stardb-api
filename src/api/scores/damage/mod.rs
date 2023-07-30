mod uid;

use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::{
    api::scores::{Scores, ScoresParams},
    database::{self, DbScoreDamage},
    Result,
};

use super::Region;

#[derive(OpenApi)]
#[openapi(
    tags((name = "scores/damage")),
    paths(get_scores_damage, put_score_damage_temporary),
    components(schemas(
        ScoreDamage,
        DamageUpdateTemporary
    ))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct DamageUpdateTemporary {
    pub uid: i64,
    pub character: String,
    pub support: bool,
    pub damage: i32,
    pub video: String,
}

#[derive(Deserialize, IntoParams)]
pub struct DamageParams {
    pub character: Option<String>,
    pub support: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct ScoreDamage {
    pub global_rank: i64,
    pub regional_rank: i64,
    pub uid: i64,
    pub character: String,
    pub support: bool,
    pub damage: i32,
    pub video: String,
    pub region: Region,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub updated_at: NaiveDateTime,
}

impl From<DbScoreDamage> for ScoreDamage {
    fn from(db_score: DbScoreDamage) -> Self {
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
    cfg.service(put_score_damage_temporary)
        .service(get_scores_damage)
        .configure(uid::configure);
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
        (status = 200, description = "ScoresDamage", body = ScoresDamage),
    )
)]
#[get("/api/scores/damage")]
async fn get_scores_damage(
    damage_params: web::Query<DamageParams>,
    scores_params: web::Query<ScoresParams>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let count_na = database::count_scores_damage(&Region::NA.to_string(), &pool).await?;
    let count_eu = database::count_scores_damage(&Region::EU.to_string(), &pool).await?;
    let count_asia = database::count_scores_damage(&Region::Asia.to_string(), &pool).await?;
    let count_cn = database::count_scores_damage(&Region::CN.to_string(), &pool).await?;

    let count = count_na + count_eu + count_asia + count_cn;

    let db_scores_damage = database::get_scores_damage(
        damage_params.character.clone(),
        damage_params.support,
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores_damage
        .into_iter()
        .map(ScoreDamage::from)
        .collect();

    let scores_damage = Scores {
        count,
        count_na,
        count_eu,
        count_asia,
        count_cn,
        scores,
    };

    Ok(HttpResponse::Ok().json(scores_damage))
}

#[utoipa::path(
    tag = "/scores/damage",
    put,
    path = "/api/scores/damage",
    request_body = DamageUpdateTemporary,
    responses(
        (status = 200, description = "ScoreDamage updated"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[put("/api/scores/damage")]
async fn put_score_damage_temporary(
    session: Session,
    damage_update_temporary: web::Json<DamageUpdateTemporary>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let uid = damage_update_temporary.uid;
    let character = damage_update_temporary.character.clone();
    let support = damage_update_temporary.support;
    let damage = damage_update_temporary.damage;
    let video = damage_update_temporary.video.clone();

    let db_set_score_damage = DbScoreDamage {
        uid,
        character,
        support,
        damage,
        video,
        ..Default::default()
    };

    database::set_score_damage(&db_set_score_damage, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
