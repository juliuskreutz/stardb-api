use actix_web::{get, put, web, HttpResponse, Responder};
use chrono::Utc;
use regex::{Captures, Regex};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{ApiResult, LanguageParams},
    database, mihomo,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "mihomo/{uid}")),
    paths(get_mihomo, put_mihomo)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_mihomo).service(put_mihomo);
}

#[utoipa::path(
    tag = "mihomo/{uid}",
    get,
    path = "/api/mihomo/{uid}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Cached mihomo json"),
    )
)]
#[get("/api/mihomo/{uid}")]
async fn get_mihomo(
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
) -> ApiResult<impl Responder> {
    let json = mihomo::get_whole(*uid, language_params.lang).await?;

    Ok(HttpResponse::Ok().json(json))
}

#[utoipa::path(
    tag = "mihomo/{uid}",
    put,
    path = "/api/mihomo/{uid}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Updated"),
    )
)]
#[put("/api/mihomo/{uid}")]
async fn put_mihomo(
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let now = Utc::now();

    let uid = *uid;

    let info = mihomo::update_and_get(uid, language_params.lang).await?;

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
    let updated_at = info.updated_at;
    let timestamp = database::get_score_achievement_by_uid(uid, &pool)
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
                "na" => chrono::Duration::try_hours(-5).unwrap(),
                "eu" => chrono::Duration::try_hours(1).unwrap(),
                _ => chrono::Duration::try_hours(8).unwrap(),
            },
        );

    let db_mihomo = database::DbMihomo {
        uid,
        region,
        name,
        level,
        signature,
        avatar_icon,
        achievement_count,
        updated_at,
    };

    database::set_mihomo(&db_mihomo, &pool).await?;

    let db_score_achievement = database::DbScoreAchievement {
        uid,
        timestamp,
        ..Default::default()
    };

    database::set_score_achievement(&db_score_achievement, &pool).await?;

    let json = mihomo::get_whole(uid, language_params.lang).await?;

    Ok(HttpResponse::Ok().json(json))
}
