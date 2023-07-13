use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::{
    api::{params::*, schemas::*},
    database::{self, DbScoreShield},
    Result,
};

impl<T: AsRef<DbScoreShield>> From<T> for ScoreShield {
    fn from(value: T) -> Self {
        let db_score = value.as_ref();

        ScoreShield {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            shield: db_score.shield,
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
    let count_na = database::count_scores_shield(&Region::NA.to_string(), &pool).await?;
    let count_eu = database::count_scores_shield(&Region::EU.to_string(), &pool).await?;
    let count_asia = database::count_scores_shield(&Region::Asia.to_string(), &pool).await?;
    let count_cn = database::count_scores_shield(&Region::CN.to_string(), &pool).await?;

    let count = count_na + count_eu + count_asia + count_cn;

    let db_scores = database::get_scores_shield(
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores.iter().map(ScoreShield::from).collect();

    let scores = Scores {
        count,
        count_na,
        count_eu,
        count_asia,
        count_cn,
        scores,
    };

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
    let score: ScoreShield = database::get_score_shield_by_uid(*uid, &pool).await?.into();

    Ok(HttpResponse::Ok().json(score))
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
    session: Session,
    uid: web::Path<i64>,
    shield_update: web::Json<ShieldUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let uid = *uid;
    let shield = shield_update.shield;
    let video = shield_update.video.clone();

    let db_score = DbScoreShield {
        uid,
        shield,
        video,
        ..Default::default()
    };

    database::set_score_shield(&db_score, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
