use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(paths(get_community_tier_list))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_community_tier_list);
}

#[derive(Serialize)]
struct CommunityTierList {
    total_votes: i32,
    entries: Vec<Entry>,
    sextiles: Vec<f64>,
}

#[derive(Serialize)]
struct Entry {
    character: i32,
    eidolon: i32,
    average: f64,
    variance: f64,
    quartile_1: f64,
    quartile_3: f64,
    confidence_interval_95: f64,
    votes: i32,
    character_rarity: i32,
    character_name: String,
    character_path: String,
    character_element: String,
    character_path_id: String,
    character_element_id: String,
}

impl From<database::DbCommunityTierListEntry> for Entry {
    fn from(db_entry: database::DbCommunityTierListEntry) -> Self {
        Entry {
            character: db_entry.character,
            eidolon: db_entry.eidolon,
            average: db_entry.average,
            variance: db_entry.variance,
            quartile_1: db_entry.quartile_1,
            quartile_3: db_entry.quartile_3,
            confidence_interval_95: db_entry.confidence_interval_95,
            votes: db_entry.votes,
            character_rarity: db_entry.character_rarity,
            character_name: db_entry.character_name,
            character_path: db_entry.character_path,
            character_element: db_entry.character_element,
            character_path_id: db_entry.character_path_id,
            character_element_id: db_entry.character_element_id,
        }
    }
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/community-tier-list",
    params(LanguageParams),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "CommunityTierList"),
    )
)]
#[get("/api/pages/community-tier-list", guard = "private")]
async fn get_community_tier_list(
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_entries =
        database::get_community_tier_list_entries(language_params.lang, &pool).await?;

    let total_votes = db_entries[0].total_votes;
    let entries: Vec<_> = db_entries.into_iter().map(Entry::from).collect();
    let sextiles = database::get_community_tier_list_sextiles(&pool)
        .await?
        .into_iter()
        .map(|s| s.value)
        .collect();

    let community_tier_list = CommunityTierList {
        total_votes,
        entries,
        sextiles,
    };

    Ok(HttpResponse::Ok().json(community_tier_list))
}
