use actix_web::{get, put, web, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDateTime;
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString};
use utoipa::{IntoParams, ToSchema};

use super::{Region, ScoresParams};
use crate::{
    api::users::Claims,
    database::{self, DbScoreDamage},
    Result,
};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ScoresDamage {
    count: i64,
    scores: Vec<ScoreDamage>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ScoreDamage {
    global_rank: i64,
    regional_rank: i64,
    uid: i64,
    character: Character,
    support: bool,
    damage: i32,
    region: Region,
    name: String,
    level: i32,
    signature: String,
    avatar_icon: String,
    updated_at: NaiveDateTime,
}

impl<T: AsRef<DbScoreDamage>> From<T> for ScoreDamage {
    fn from(value: T) -> Self {
        let db_score = value.as_ref();

        ScoreDamage {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            character: db_score.character.parse().unwrap(),
            support: db_score.support,
            damage: db_score.damage,
            region: db_score.region.parse().unwrap(),
            name: db_score.name.clone(),
            level: db_score.level,
            signature: db_score.signature.clone(),
            avatar_icon: db_score.avatar_icon.clone(),
            updated_at: db_score.updated_at,
        }
    }
}

#[derive(Display, EnumString, Serialize, Deserialize, ToSchema)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Character {
    Seele,
    Yanqing,
    JingYuan,
    QingQue,
}

#[derive(Deserialize, IntoParams)]
struct DamageParams {
    character: Option<Character>,
    support: Option<bool>,
}

#[utoipa::path(
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
    let count = database::count_scores_damage(&pool).await?;

    let db_scores_damage = database::get_scores_damage(
        damage_params.character.as_ref().map(|c| c.to_string()),
        damage_params.support,
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores_damage.iter().map(ScoreDamage::from).collect();

    let scores_damage = ScoresDamage { count, scores };

    Ok(HttpResponse::Ok().json(scores_damage))
}

#[utoipa::path(
    get,
    path = "/api/scores/damage/{uid}",
    responses(
        (status = 200, description = "[ScoreDamage]", body = Vec<ScoreDamage>),
    )
)]
#[get("/api/scores/damage/{uid}")]
async fn get_score_damage(uid: web::Path<i64>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let scores: Vec<_> = database::get_scores_damage_by_uid(*uid, &pool)
        .await?
        .iter()
        .map(ScoreDamage::from)
        .collect();

    Ok(HttpResponse::Ok().json(scores))
}

#[derive(Deserialize, ToSchema)]
pub struct DamageUpdate {
    character: Character,
    support: bool,
    damage: i32,
}

#[utoipa::path(
    put,
    path = "/api/scores/damage/{uid}",
    request_body = DamageUpdate,
    responses(
        (status = 200, description = "ScoreDamage updated"),
        (status = 403, description = "Not an admin"),
    )
)]
#[put("/api/scores/damage/{uid}")]
async fn put_score_damage(
    request: HttpRequest,
    uid: web::Path<i64>,
    damage_update: web::Json<DamageUpdate>,
    jwt_secret: web::Data<[u8; 32]>,

    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Some(cookie) = request.cookie("token") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let claims: Claims = jsonwebtoken::decode(
        cookie.value(),
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|t| t.claims)?;

    if !claims.admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let uid = *uid;
    let character = damage_update.character.to_string();
    let support = damage_update.support;
    let damage = damage_update.damage;

    let db_set_score_damage = DbScoreDamage {
        uid,
        character,
        support,
        damage,
        ..Default::default()
    };

    database::set_score_damage(&db_set_score_damage, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
