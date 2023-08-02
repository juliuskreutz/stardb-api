use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::{self, DbTierListEntry},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "tier-list/{id}")),
    paths(get_tier_list_entry, put_tier_list_entry),
    components(schemas(
        TierListEntryUpdate
    ))
)]
struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct TierListEntryUpdate {
    eidolon: i32,
    role: String,
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
    score_number: i32,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_tier_list_entry)
        .service(put_tier_list_entry);
}

#[utoipa::path(
    tag = "tier-list/{id}",
    get,
    path = "/api/tier-list/{id}",
    responses(
        (status = 200, description = "[TierListEntry]", body = Vec<TierListEntry>),
    )
)]
#[get("/api/tier-list/{id}")]
async fn get_tier_list_entry(
    _id: web::Path<i32>,
    _pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    //TODO: Implement me

    Ok(HttpResponse::Ok())
}

#[utoipa::path(
    tag = "tier-list/{id}",
    put,
    path = "/api/tier-list/{id}",
    responses(
        (status = 200, description = "Success"),
    ),
    security(("admin" = []))
)]
#[put("/api/tier-list/{id}")]
async fn put_tier_list_entry(
    session: Session,
    id: web::Path<i32>,
    tier_list_entry_update: web::Json<TierListEntryUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let db_tier_list_entry = DbTierListEntry {
        character: *id,
        character_tag: String::new(),
        character_name: String::new(),
        character_element: String::new(),
        character_path: String::new(),
        eidolon: tier_list_entry_update.eidolon,
        role: tier_list_entry_update.role.clone(),
        st_dps: tier_list_entry_update.st_dps,
        aoe_dps: tier_list_entry_update.aoe_dps,
        buffer: tier_list_entry_update.buffer,
        debuffer: tier_list_entry_update.debuffer,
        healer: tier_list_entry_update.healer,
        survivability: tier_list_entry_update.survivability,
        sp_efficiency: tier_list_entry_update.sp_efficiency,
        avg_break: tier_list_entry_update.avg_break,
        st_break: tier_list_entry_update.st_break,
        base_speed: tier_list_entry_update.base_speed,
        footnote: tier_list_entry_update.footnote.clone(),
        score: tier_list_entry_update.score,
        score_number: tier_list_entry_update.score_number,
    };

    database::set_tier_list_entry(&db_tier_list_entry, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
