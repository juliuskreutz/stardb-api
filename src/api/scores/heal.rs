use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;

use crate::{
    api::{params::*, schemas::*},
    database::{self, DbScoreHeal},
    Result,
};

impl<T: AsRef<DbScoreHeal>> From<T> for ScoreHeal {
    fn from(value: T) -> Self {
        let db_score = value.as_ref();

        ScoreHeal {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            heal: db_score.heal,
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

    let db_scores = database::get_scores_heal(
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores.iter().map(ScoreHeal::from).collect();

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
    path = "/api/scores/heal/{uid}",
    responses(
        (status = 200, description = "[ScoreHeal]", body = ScoreHeal),
    )
)]
#[get("/api/scores/heal/{uid}")]
async fn get_score_heal(uid: web::Path<i64>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let score: ScoreHeal = database::get_score_heal_by_uid(*uid, &pool).await?.into();

    Ok(HttpResponse::Ok().json(score))
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
    session: Session,
    uid: web::Path<i64>,
    heal_update: web::Json<HealUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let uid = *uid;
    let heal = heal_update.heal;
    let video = heal_update.video.clone();

    let db_score = DbScoreHeal {
        uid,
        heal,
        video,
        ..Default::default()
    };

    database::set_score_heal(&db_score, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
