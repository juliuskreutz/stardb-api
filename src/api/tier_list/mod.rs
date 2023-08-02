mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::{self, DbTierListEntry},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "tier-list")),
    paths(get_tier_list),
    components(schemas(
        TierListEntry
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
pub struct TierListEntry {
    character: i32,
    character_tag: String,
    character_name: String,
    character_element: String,
    character_path: String,
    eidolon: i32,
    role: Option<String>,
    st_dps: Option<i32>,
    aoe_dps: Option<i32>,
    buffer: Option<i32>,
    debuffer: Option<i32>,
    healer: Option<i32>,
    survivability: Option<i32>,
    sp_efficiency: Option<i32>,
    avg_break: Option<i32>,
    st_break: Option<i32>,
    base_speed: Option<i32>,
    footnote: Option<String>,
    score: i32,
}

impl From<DbTierListEntry> for TierListEntry {
    fn from(db_tier_list_entry: DbTierListEntry) -> Self {
        Self {
            character: db_tier_list_entry.character,
            character_tag: db_tier_list_entry.character_tag,
            character_name: db_tier_list_entry.character_name,
            character_element: db_tier_list_entry.character_element,
            character_path: db_tier_list_entry.character_path,
            eidolon: db_tier_list_entry.eidolon,
            role: db_tier_list_entry.role,
            st_dps: db_tier_list_entry.st_dps,
            aoe_dps: db_tier_list_entry.aoe_dps,
            buffer: db_tier_list_entry.buffer,
            debuffer: db_tier_list_entry.debuffer,
            healer: db_tier_list_entry.healer,
            survivability: db_tier_list_entry.survivability,
            sp_efficiency: db_tier_list_entry.sp_efficiency,
            avg_break: db_tier_list_entry.avg_break,
            st_break: db_tier_list_entry.st_break,
            base_speed: db_tier_list_entry.base_speed,
            footnote: db_tier_list_entry.footnote,
            score: db_tier_list_entry.score,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_tier_list).configure(id::configure);
}

#[utoipa::path(
    tag = "tier-list",
    get,
    path = "/api/tier-list",
    responses(
        (status = 200, description = "[TierListEntry]", body = Vec<TierListEntry>),
    )
)]
#[get("/api/tier-list")]
async fn get_tier_list(pool: web::Data<PgPool>) -> Result<impl Responder> {
    let tier_list_entries: Vec<_> = database::get_tier_list_entries(&pool)
        .await?
        .into_iter()
        .map(TierListEntry::from)
        .collect();

    Ok(HttpResponse::Ok().json(tier_list_entries))
}
