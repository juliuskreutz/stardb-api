use actix_session::Session;
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

impl From<database::warps::DbCharacterCount> for Character {
    fn from(db_character: database::warps::DbCharacterCount) -> Self {
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

impl From<database::warps::DbLightConeCount> for LightCone {
    fn from(db_light_cone: database::warps::DbLightConeCount) -> Self {
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
    session: Session,
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    let mut forbidden = database::get_connections_by_uid(uid, &pool)
        .await?
        .iter()
        .any(|c| c.private);

    if forbidden {
        if let Ok(Some(username)) = session.get::<String>("username") {
            if let Ok(connection) =
                database::get_connection_by_uid_and_username(uid, &username, &pool).await
            {
                forbidden = !connection.verified;
            }
        }
    }

    if forbidden {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let profile = get_profile_json(false, uid, language_params.lang, &pool).await?;

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
    session: Session,
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    let mut forbidden = database::get_connections_by_uid(uid, &pool)
        .await?
        .iter()
        .any(|c| c.private);

    if forbidden {
        if let Ok(Some(username)) = session.get::<String>("username") {
            if let Ok(connection) =
                database::get_connection_by_uid_and_username(uid, &username, &pool).await
            {
                forbidden = !connection.verified;
            }
        }
    }

    if forbidden {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let profile = get_profile_json(true, uid, language_params.lang, &pool).await?;

    Ok(HttpResponse::Ok().json(profile))
}

async fn get_profile_json(
    update: bool,
    uid: i32,
    lang: Language,
    pool: &PgPool,
) -> ApiResult<Profile> {
    let mihomo = if update {
        mihomo::update_and_get(uid, lang, pool).await?
    } else {
        mihomo::get(uid, lang, pool).await?
    };

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

    let character_counts = database::warps::get_characters_count_by_uid(uid, lang, pool).await?;
    let light_cones_counts = database::warps::get_light_cones_count_by_uid(uid, lang, pool).await?;

    let departure = database::warps::departure::get_count_by_uid(uid, pool).await?;
    let standard = database::warps::standard::get_count_by_uid(uid, pool).await?;
    let special = database::warps::special::get_count_by_uid(uid, pool).await?;
    let lc = database::warps::lc::get_count_by_uid(uid, pool).await?;
    let total = departure + standard + special + lc;

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
