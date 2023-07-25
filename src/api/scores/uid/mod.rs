use actix_web::{get, put, web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use regex::{Captures, Regex};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::schemas::ScoreAchievement,
    database::{self, DbScore},
    mihomo, Result,
};

#[derive(OpenApi)]
#[openapi(tags((name = "scores/{uid}")), paths(get_score_achievement, put_score_achievement))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_score_achievement)
        .service(put_score_achievement);
}

#[utoipa::path(
    tag = "scores/{uid}",
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
    tag = "scores/{uid}",
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
