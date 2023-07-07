use actix_web::{get, put, web, HttpRequest, HttpResponse, Responder};
use chrono::NaiveDateTime;
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use utoipa::ToSchema;

use super::{Region, ScoresParams};
use crate::{
    api::users::Claims,
    database::{self, DbScoreHeal},
    Result,
};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ScoresHeal {
    count: i64,
    scores: Vec<ScoreHealPartial>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ScoreHealPartial {
    global_rank: i64,
    regional_rank: i64,
    uid: i64,
    heal: i32,
    region: Region,
    name: String,
    level: u64,
    signature: String,
    avatar_icon: String,
    updated_at: NaiveDateTime,
}

impl<T: AsRef<DbScoreHeal>> From<T> for ScoreHealPartial {
    fn from(value: T) -> Self {
        let db_score = value.as_ref();

        let name = db_score.info["player"]["nickname"]
            .as_str()
            .unwrap()
            .to_string();

        let level = db_score.info["player"]["level"].as_u64().unwrap();

        let signature = db_score.info["player"]["signature"]
            .as_str()
            .unwrap()
            .to_string();

        let avatar_icon = db_score.info["player"]["avatar"]["icon"]
            .as_str()
            .unwrap()
            .to_string();

        ScoreHealPartial {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            heal: db_score.heal,
            region: db_score.region.parse().unwrap(),
            name,
            level,
            signature,
            avatar_icon,
            updated_at: db_score.updated_at,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ScoreHeal {
    global_rank: i64,
    regional_rank: i64,
    uid: i64,
    region: Region,
    info: Value,
    updated_at: NaiveDateTime,
}

impl<T: AsRef<DbScoreHeal>> From<T> for ScoreHeal {
    fn from(value: T) -> Self {
        let db_score = value.as_ref();

        Self {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            region: db_score.region.parse().unwrap(),
            info: db_score.info.clone(),
            updated_at: db_score.updated_at,
        }
    }
}

#[utoipa::path(
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
    let count = database::count_scores_heal(&pool).await?;

    let db_scores = database::get_scores_heal(
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores.iter().map(ScoreHealPartial::from).collect();

    let scores = ScoresHeal { count, scores };

    Ok(HttpResponse::Ok().json(scores))
}

#[utoipa::path(
    get,
    path = "/api/scores/heal/{uid}",
    responses(
        (status = 200, description = "ScoreHeal", body = ScoreHeal),
    )
)]
#[get("/api/scores/heal/{uid}")]
async fn get_score_heal(uid: web::Path<i64>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let scores: ScoreHeal = database::get_score_heal_by_uid(*uid, &pool).await?.into();

    Ok(HttpResponse::Ok().json(scores))
}

#[derive(Deserialize, ToSchema)]
pub struct HealUpdate {
    heal: i32,
}

#[utoipa::path(
    put,
    path = "/api/scores/heal/{uid}",
    request_body = HealUpdate,
    responses(
        (status = 200, description = "ScoreHeal updated"),
        (status = 403, description = "Not an admin"),
    )
)]
#[put("/api/scores/heal/{uid}")]
async fn put_score_heal(
    request: HttpRequest,
    uid: web::Path<i64>,
    heal_update: web::Json<HealUpdate>,
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
    let heal = heal_update.heal;

    let db_score = DbScoreHeal {
        uid,
        heal,
        ..Default::default()
    };

    database::set_score_heal(&db_score, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
