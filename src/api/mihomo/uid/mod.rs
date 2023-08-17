use std::{fs::File, path::PathBuf};

use actix_web::{get, put, web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use regex::{Captures, Regex};
use serde_json::Value;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, mihomo};

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
    responses(
        (status = 200, description = "Cached mihomo json"),
    )
)]
#[get("/api/mihomo/{uid}")]
async fn get_mihomo(uid: web::Path<i64>) -> ApiResult<impl Responder> {
    let path = format!("mihomo/{uid}.json");

    if !PathBuf::from(&path).exists() {
        reqwest::Client::new()
            .put(&format!("http://localhost:8000/api/mihomo/{uid}"))
            .send()
            .await?;
    }

    let json: Value = serde_json::from_reader(File::open(&path)?)?;

    Ok(HttpResponse::Ok().json(json))
}

#[utoipa::path(
    tag = "mihomo/{uid}",
    put,
    path = "/api/mihomo/{uid}",
    responses(
        (status = 200, description = "Updated"),
    )
)]
#[put("/api/mihomo/{uid}")]
async fn put_mihomo(uid: web::Path<i64>, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
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
    let updated_at = info.updated_at;
    let timestamp = stardb_database::get_score_achievement_by_uid(uid, &pool)
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

    let db_mihomo = stardb_database::DbMihomo {
        uid,
        region,
        name,
        level,
        signature,
        avatar_icon,
        achievement_count,
        updated_at,
    };

    stardb_database::set_mihomo(&db_mihomo, &pool).await?;

    let db_score_achievement = stardb_database::DbScoreAchievement {
        uid,
        timestamp,
        ..Default::default()
    };

    stardb_database::set_score_achievement(&db_score_achievement, &pool).await?;

    Ok(HttpResponse::Ok())
}
