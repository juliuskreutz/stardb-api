use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

use super::LanguageParams;

#[derive(OpenApi)]
#[openapi(
    tags((name = "community-tier-list")),
    paths(get_community_tier_list_entries),
    components(schemas(
        CommunityTierListEntry
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct CommunityTierListEntry {
    character: i32,
    character_name: String,
    character_path: String,
    character_element: String,
    character_path_id: String,
    character_element_id: String,
    eidolon: i32,
    average: f64,
    variance: f64,
    quartile_1: f64,
    quartile_3: f64,
    confidence_interval_95: f64,
    votes: i32,
}

impl From<database::DbCommunityTierListEntry> for CommunityTierListEntry {
    fn from(db_entry: database::DbCommunityTierListEntry) -> Self {
        CommunityTierListEntry {
            character: db_entry.character,
            character_name: db_entry.character_name,
            character_path: db_entry.character_path,
            character_element: db_entry.character_element,
            character_path_id: db_entry.character_path_id,
            character_element_id: db_entry.character_element_id,
            eidolon: db_entry.eidolon,
            average: db_entry.average,
            variance: db_entry.variance,
            quartile_1: db_entry.quartile_1,
            quartile_3: db_entry.quartile_3,
            confidence_interval_95: db_entry.confidence_interval_95,
            votes: db_entry.votes,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_community_tier_list_entries);
}

#[utoipa::path(
    tag = "community-tier-list",
    get,
    path = "/api/community-tier-list",
    params(LanguageParams),
    responses(
        (status = 200, description = "[CommunityTierListEntry]", body = Vec<CommunityTierListEntry>),
    )
)]
#[get("/api/community-tier-list")]
async fn get_community_tier_list_entries(
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_community_tier_list_entries =
        database::get_community_tier_list_entries(language_params.lang, &pool).await?;

    let community_tier_list_entries: Vec<_> = db_community_tier_list_entries
        .into_iter()
        .map(CommunityTierListEntry::from)
        .collect();

    Ok(HttpResponse::Ok().json(community_tier_list_entries))
}
