use actix_web::{get, put, web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use regex::{Captures, Regex};
use sqlx::PgPool;

use crate::{
    api::{params::*, schemas::*},
    database::{self, DbScore},
    mihomo, Result,
};

pub mod damage;
pub mod heal;
pub mod shield;

impl<T: AsRef<DbScore>> From<T> for ScoreAchievement {
    fn from(value: T) -> Self {
        let db_score = value.as_ref();

        ScoreAchievement {
            global_rank: db_score.global_rank.unwrap(),
            regional_rank: db_score.regional_rank.unwrap(),
            uid: db_score.uid,
            region: db_score.region.parse().unwrap(),
            name: db_score.name.clone(),
            level: db_score.level,
            signature: db_score.signature.clone(),
            avatar_icon: db_score.avatar_icon.clone(),
            achievement_count: db_score.achievement_count,
            updated_at: db_score.updated_at,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/scores",
    params(
        ScoresParams
    ),
    responses(
        (status = 200, description = "ScoresAchievement", body = ScoresAchievement),
    )
)]
#[get("/api/scores")]
async fn get_scores_achievement(
    scores_params: web::Query<ScoresParams>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let count_na = database::count_scores(&Region::NA.to_string(), &pool).await?;
    let count_eu = database::count_scores(&Region::EU.to_string(), &pool).await?;
    let count_asia = database::count_scores(&Region::Asia.to_string(), &pool).await?;
    let count_cn = database::count_scores(&Region::CN.to_string(), &pool).await?;

    let count = count_na + count_eu + count_asia + count_cn;

    let db_scores = database::get_scores(
        scores_params.region.as_ref().map(|r| r.to_string()),
        scores_params.query.clone(),
        scores_params.limit,
        scores_params.offset,
        &pool,
    )
    .await?;

    let scores = db_scores.iter().map(ScoreAchievement::from).collect();

    let scores_achievement = Scores {
        count,
        count_na,
        count_eu,
        count_asia,
        count_cn,
        scores,
    };

    Ok(HttpResponse::Ok().json(scores_achievement))
}

#[utoipa::path(
    get,
    path = "/api/scores/{uid}",
    responses(
        (status = 200, description = "ScoreAchievement", body = ScoreAchievement),
    )
)]
#[get("/api/scores/{uid}")]
async fn get_score_achievement(
    uid: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let score: ScoreAchievement = database::get_score_by_uid(*uid, &pool).await?.into();

    Ok(HttpResponse::Ok().json(score))
}

#[utoipa::path(
    put,
    path = "/api/scores/{uid}",
    responses(
        (status = 200, description = "ScoreAchievement updated", body = ScoreAchievement),
    )
)]
#[put("/api/scores/{uid}")]
async fn put_score_achievement(
    uid: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let now = Utc::now().naive_utc();

    let uid = *uid;

    let info = mihomo::get(uid).await?;

    let re = Regex::new(r"<[^>]*>")?;

    let name = re
        .replace_all(&info.player.nickname, |_: &Captures| "")
        .to_string();
    let region = match uid.to_string().chars().next() {
        Some('6') => "na",
        Some('7') => "eu",
        Some('8') | Some('9') => "asia",
        _ => "cn",
    }
    .to_string();
    let level = info.player.level;
    let avatar_icon = info.player.avatar.icon.clone();
    let signature = re
        .replace_all(&info.player.signature, |_: &Captures| "")
        .to_string();
    let achievement_count = info.player.space_info.achievement_count;
    let timestamp = database::get_score_by_uid(uid, &pool)
        .await
        .ok()
        .and_then(|sd| {
            if sd.achievement_count == achievement_count {
                Some(sd.timestamp)
            } else {
                None
            }
        })
        .unwrap_or(
            now + match region.as_str() {
                "na" => Duration::hours(-5),
                "eu" => Duration::hours(1),
                _ => Duration::hours(8),
            },
        );

    let db_score = DbScore {
        uid,
        region,
        name,
        level,
        signature,
        avatar_icon,
        achievement_count,
        timestamp,
        ..Default::default()
    };

    let score: ScoreAchievement = database::set_score(&db_score, &pool).await?.into();

    Ok(HttpResponse::Ok().json(score))
}
