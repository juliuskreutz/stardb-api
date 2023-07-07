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
    database::{self, DbScoreShield},
    Result,
};

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ScoresShield {
    count: i64,
    scores: Vec<ScoreShieldPartial>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ScoreShieldPartial {
    global_rank: i64,
    regional_rank: i64,
    uid: i64,
    shield: i32,
    region: Region,
    name: String,
    level: u64,
    signature: String,
    avatar_icon: String,
    updated_at: NaiveDateTime,
}

impl<T: AsRef<DbScoreShield>> From<T> for ScoreShieldPartial {
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

        ScoreShieldPartial {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            shield: db_score.shield,
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
pub struct ScoreShield {
    global_rank: i64,
    regional_rank: i64,
    uid: i64,
    region: Region,
    info: Value,
    updated_at: NaiveDateTime,
}

impl<T: AsRef<DbScoreShield>> From<T> for ScoreShield {
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
    let count = database::count_scores_shield(&pool).await?;

    let db_scores = database::get_scores_shield(
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores.iter().map(ScoreShieldPartial::from).collect();

    let scores = ScoresShield { count, scores };

    Ok(HttpResponse::Ok().json(scores))
}

#[utoipa::path(
    get,
    path = "/api/scores/shield/{uid}",
    responses(
        (status = 200, description = "ScoreShield", body = ScoreShield),
    )
)]
#[get("/api/scores/shield/{uid}")]
async fn get_score_shield(uid: web::Path<i64>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let scores: ScoreShield = database::get_score_shield_by_uid(*uid, &pool).await?.into();

    Ok(HttpResponse::Ok().json(scores))
}

#[derive(Deserialize, ToSchema)]
pub struct ShieldUpdate {
    shield: i32,
}

#[utoipa::path(
    put,
    path = "/api/scores/shield/{uid}",
    request_body = ShieldUpdate,
    responses(
        (status = 200, description = "ScoreShield updated"),
        (status = 403, description = "Not an admin"),
    )
)]
#[put("/api/scores/shield/{uid}")]
async fn put_score_shield(
    request: HttpRequest,
    uid: web::Path<i64>,
    shield_update: web::Json<ShieldUpdate>,
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
    let shield = shield_update.shield;

    let db_score = DbScoreShield {
        uid,
        shield,
        ..Default::default()
    };

    database::set_score_shield(&db_score, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
