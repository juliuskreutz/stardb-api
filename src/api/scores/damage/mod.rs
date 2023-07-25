mod uid;

use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{params::*, schemas::*},
    database::{self, DbScoreDamage},
    Result,
};

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

impl<T: AsRef<DbScoreDamage>> From<T> for ScoreDamage {
    fn from(value: T) -> Self {
        let db_score = value.as_ref();

        ScoreDamage {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            character: db_score.character.clone(),
            support: db_score.support,
            damage: db_score.damage,
            video: db_score.video.clone(),
            region: db_score.region.parse().unwrap(),
            name: db_score.name.clone(),
            level: db_score.level,
            signature: db_score.signature.clone(),
            avatar_icon: db_score.avatar_icon.clone(),
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

    let scores = db_scores_damage.iter().map(ScoreDamage::from).collect();

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
    tag = "pinned",
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
