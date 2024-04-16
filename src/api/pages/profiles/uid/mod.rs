use actix_web::{get, put, web, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, LanguageParams, Region},
    database, mihomo, Language,
};

#[derive(OpenApi)]
#[openapi(paths(get_profile, update_profile))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_profile).service(update_profile);
}

#[derive(Serialize)]
struct Profile {
    rank_global: i64,
    rank_regional: i64,
    top_global: f64,
    top_regional: f64,
    region: Region,
    updated_at: DateTime<Utc>,
    mihomo: Value,
    collection: Collection,
}

#[derive(Serialize)]
struct Collection {
    total: i64,
    departure: i64,
    standard: i64,
    special: i64,
    lc: i64,
    characters: Vec<Character>,
    light_cones: Vec<LightCone>,
}

#[derive(Serialize)]
struct Character {
    id: i32,
    rarity: i32,
    name: String,
    path: String,
    element: String,
    path_id: String,
    element_id: String,
    count: i64,
}

#[derive(Serialize)]
struct LightCone {
    id: i32,
    rarity: i32,
    name: String,
    path: String,
    path_id: String,
    count: i64,
}

impl From<database::DbCharacterCount> for Character {
    fn from(db_character: database::DbCharacterCount) -> Self {
        Character {
            id: db_character.id,
            rarity: db_character.rarity,
            name: db_character.name,
            path: db_character.path,
            element: db_character.element,
            path_id: db_character.path_id,
            element_id: db_character.element_id,
            count: db_character.count.unwrap_or_default(),
        }
    }
}

impl From<database::DbLightConeCount> for LightCone {
    fn from(db_light_cone: database::DbLightConeCount) -> Self {
        LightCone {
            id: db_light_cone.id,
            rarity: db_light_cone.rarity,
            name: db_light_cone.name,
            path: db_light_cone.path,
            path_id: db_light_cone.path_id,
            count: db_light_cone.count.unwrap_or_default(),
        }
    }
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/profiles/{uid}",
    params(LanguageParams),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "Profile"),
    )
)]
#[get("/api/pages/profiles/{uid}", guard = "private")]
async fn get_profile(
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let profile = get_profile_json(*uid, language_params.lang, &pool).await?;

    Ok(HttpResponse::Ok().json(profile))
}

#[utoipa::path(
    tag = "pages",
    put,
    path = "/api/pages/profiles/{uid}",
    params(LanguageParams),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "Profile"),
    )
)]
#[put("/api/pages/profiles/{uid}", guard = "private")]
async fn update_profile(
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    reqwest::Client::new()
        .put(format!("http://localhost:8000/api/mihomo/{uid}"))
        .send()
        .await?;

    let profile = get_profile_json(*uid, language_params.lang, &pool).await?;

    Ok(HttpResponse::Ok().json(profile))
}

async fn get_profile_json(uid: i32, lang: Language, pool: &PgPool) -> ApiResult<Profile> {
    let mihomo = mihomo::get_whole(uid, lang).await?;

    let score_achievement = database::get_score_achievement_by_uid(uid, pool).await?;

    let rank_global = score_achievement.global_rank.unwrap_or_default();
    let rank_regional = score_achievement.regional_rank.unwrap_or_default();

    let count_global = database::count_scores_achievement(None, None, pool).await?;
    let count_regional =
        database::count_scores_achievement(Some(&score_achievement.region), None, pool).await?;

    let top_global = rank_global as f64 / count_global as f64;
    let top_regional = rank_regional as f64 / count_regional as f64;

    let region = score_achievement.region.parse()?;

    let updated_at = score_achievement.updated_at;

    let warps_count = database::get_warp_counts_by_uid(uid, pool).await?;
    let character_counts =
        database::get_characters_count_by_uid(uid, &lang.to_string(), pool).await?;
    let light_cones_counts =
        database::get_light_cones_count_by_uid(uid, &lang.to_string(), pool).await?;

    let mut total = 0;
    let mut departure = 0;
    let mut standard = 0;
    let mut special = 0;
    let mut lc = 0;
    for wc in &warps_count {
        let count = wc.count.unwrap_or_default();
        total += count;
        match wc.gacha_type.as_str() {
            "departure" => departure = count,
            "standard" => standard = count,
            "special" => special = count,
            "lc" => lc = count,
            _ => {}
        }
    }

    let characters = character_counts.into_iter().map(From::from).collect();

    let light_cones = light_cones_counts.into_iter().map(From::from).collect();

    let collection = Collection {
        total,
        departure,
        standard,
        special,
        lc,
        characters,
        light_cones,
    };

    let profile = Profile {
        rank_global,
        rank_regional,
        top_global,
        top_regional,
        updated_at,
        region,
        mihomo,
        collection,
    };

    Ok(profile)
}
