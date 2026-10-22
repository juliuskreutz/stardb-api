//! NTE tracker routes, ownership checks, and validation of exported pull history.
//! Mutating routes enforce origin and session checks before accepting client records;
//! public page annotations never grant mutation authority.

use actix_session::Session;
use actix_web::{
    delete, get, http::StatusCode, post, put, web, HttpRequest, HttpResponse, Responder,
};
use chrono::{DateTime, Utc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

const TRACKER_UID_LEN: usize = 12;
const TRACKER_UID_PREFIX: &str = "21";
const TRACKER_IMPORT_MAX_BODY_BYTES: usize = 10 * 1024 * 1024;
const TRACKER_IMPORT_MAX_EXPORTS: usize = 10;
const TRACKER_IMPORT_MAX_RECORDS: usize = 10_000;
const TRACKER_MAX_STORED_RECORDS: i64 = 100_000;
const TRACKER_MAX_SELF_CLAIMS: i64 = 3;
const TRACKER_RECORD_UID_MAX_CHARS: usize = 128;
const TRACKER_TEXT_MAX_CHARS: usize = 64;
const TRACKER_REWARD_TEXT_MAX_CHARS: usize = 128;
const TRACKER_NICKNAME_MAX_CHARS: usize = 24;

#[derive(OpenApi)]
#[openapi(
    tags((name = "ntehelper/tracker")),
    paths(
        get_tracker_uids_me,
        post_tracker_uid_claim,
        put_tracker_uid_claim,
        delete_tracker_uid_claim,
        get_tracker_uid,
        post_tracker_import,
        delete_tracker_pulls,
    ),
    components(schemas(
        TrackerClaimRequest,
        TrackerClaimUpdateRequest,
        TrackerClaimsResponse,
        TrackerImportRequest,
        TrackerImportResponse,
        TrackerPullResponse,
        TrackerUidClaimResponse,
        TrackerUidDeleteResponse,
        TrackerUidPageResponse,
    ))
)]
struct ApiDoc;

/// Returns the tracker route and payload definitions for the combined OpenAPI document.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers tracker claim, public profile, and pull-import routes.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_tracker_uids_me)
        .service(post_tracker_uid_claim)
        .service(put_tracker_uid_claim)
        .service(delete_tracker_uid_claim)
        .service(get_tracker_uid)
        .service(post_tracker_import)
        .service(delete_tracker_pulls);
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TrackerClaimRequest {
    uid: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TrackerClaimsResponse {
    claims: Vec<TrackerUidClaimResponse>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TrackerUidClaimResponse {
    uid: String,
    nickname: String,
    region: String,
    owner_username: Option<String>,
    claim_source: String,
    has_profile: bool,
    claimed_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TrackerClaimUpdateRequest {
    nickname: Option<String>,
    region: Option<String>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TrackerUidPageResponse {
    claim: Option<TrackerUidClaimResponse>,
    pulls: Vec<TrackerPullResponse>,
    viewer_can_manage: bool,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TrackerUidDeleteResponse {
    deleted_pulls: i64,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TrackerPullResponse {
    uid: String,
    record_uid: String,
    pool_group_id: String,
    timestamp_raw: String,
    timestamp_group_ordinal: i32,
    reward_id: String,
    roll_result: Option<i32>,
    result_type: Option<String>,
    quantity: Option<i32>,
    source_type: Option<String>,
    imported_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TrackerImportRequest {
    exports: Vec<RawTrackerExport>,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TrackerImportResponse {
    uid: String,
    received: usize,
    imported: i64,
    updated: i64,
    skipped_duplicate: i64,
    inserted: i64,
    duplicates: i64,
    total: i64,
}

#[derive(Deserialize, ToSchema)]
struct RawTrackerExport {
    format: String,
    #[serde(rename = "format_version")]
    format_version: i32,
    user_uid: String,
    banner: RawTrackerBanner,
    records: Vec<RawTrackerRecord>,
}

#[derive(Deserialize, ToSchema)]
struct RawTrackerBanner {
    id: String,
}

#[derive(Deserialize, ToSchema)]
struct RawTrackerRecord {
    uid: String,
    pool_group_id: String,
    timestamp: String,
    timestamp_group_ordinal: i32,
    roll_result: Option<i32>,
    result_type: Option<String>,
    #[serde(rename = "reward_type")]
    _reward_type: Option<String>,
    reward_id: String,
    #[serde(rename = "reward_name")]
    _reward_name: Option<String>,
    #[serde(rename = "reward_rank")]
    _reward_rank: Option<String>,
    quantity: Option<i32>,
}

#[utoipa::path(
    tag = "ntehelper/tracker",
    get,
    path = "/api/ntehelper/tracker/uids/me",
    responses(
        (status = 200, description = "Current user's claimed NTE tracker UIDs", body = TrackerClaimsResponse),
        (status = 400, description = "Not logged in"),
        (status = 403, description = "Invalid origin"),
    )
)]
#[get("/api/ntehelper/tracker/uids/me")]
/// Lists the authenticated user's attached tracker UIDs and the self-claim cap.
async fn get_tracker_uids_me(
    request: HttpRequest,
    session: Session,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !super::valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Some(username) = current_username(&session) else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    let user_id = database::ntehelper_tracker::get_tracker_user_id(&username, &pool).await?;
    let claims = database::ntehelper_tracker::tracker_claims_for_user(user_id, &pool)
        .await?
        .into_iter()
        .map(TrackerUidClaimResponse::from)
        .collect();

    Ok(HttpResponse::Ok().json(TrackerClaimsResponse { claims }))
}

#[utoipa::path(
    tag = "ntehelper/tracker",
    post,
    path = "/api/ntehelper/tracker/uids/claim",
    request_body = TrackerClaimRequest,
    responses(
        (status = 200, description = "Claimed tracker UID", body = TrackerUidClaimResponse),
        (status = 400, description = "Not logged in or invalid UID"),
        (status = 403, description = "Invalid origin"),
        (status = 409, description = "UID already claimed or self-claim limit reached"),
    )
)]
#[post("/api/ntehelper/tracker/uids/claim")]
/// Claims a validated UID for the current user, mapping ownership/cap errors to HTTP responses.
async fn post_tracker_uid_claim(
    request: HttpRequest,
    session: Session,
    body: web::Json<TrackerClaimRequest>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !super::valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Some(username) = current_username(&session) else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    let Some(uid) = validate_tracker_uid(&body.uid) else {
        return Ok(HttpResponse::BadRequest().body("Invalid tracker UID"));
    };

    let user_id = database::ntehelper_tracker::get_tracker_user_id(&username, &pool).await?;
    let claim = match database::ntehelper_tracker::claim_tracker_uid(
        user_id,
        uid,
        TRACKER_MAX_SELF_CLAIMS,
        &pool,
    )
    .await?
    {
        Ok(claim) => claim,
        Err(error) => return Ok(HttpResponse::Conflict().body(error.to_string())),
    };

    Ok(HttpResponse::Ok().json(TrackerUidClaimResponse::from(claim)))
}

#[utoipa::path(
    tag = "ntehelper/tracker",
    put,
    path = "/api/ntehelper/tracker/uids/{uid}",
    request_body = TrackerClaimUpdateRequest,
    responses(
        (status = 200, description = "Updated claimed tracker UID nickname", body = TrackerUidClaimResponse),
        (status = 400, description = "Not logged in or invalid UID"),
        (status = 403, description = "Invalid origin"),
        (status = 409, description = "UID cannot be updated for this account"),
    )
)]
#[put("/api/ntehelper/tracker/uids/{uid}")]
/// Updates validated nickname/region fields after origin and session checks.
async fn put_tracker_uid_claim(
    request: HttpRequest,
    session: Session,
    uid: web::Path<String>,
    body: web::Json<TrackerClaimUpdateRequest>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !super::valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Some(username) = current_username(&session) else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    let Some(uid) = validate_tracker_uid(&uid) else {
        return Ok(HttpResponse::BadRequest().body("Invalid tracker UID"));
    };
    let nickname = match body.nickname.as_deref() {
        Some(value) => match validate_tracker_nickname(value) {
            Some(value) => Some(value),
            None => return Ok(HttpResponse::BadRequest().body("Invalid tracker nickname")),
        },
        None => None,
    };
    let region = match body.region.as_deref() {
        Some(value) => match validate_tracker_region(value) {
            Some(value) => Some(value),
            None => return Ok(HttpResponse::BadRequest().body("Invalid tracker region")),
        },
        None => None,
    };
    if nickname.is_none() && region.is_none() {
        return Ok(HttpResponse::BadRequest().body("Tracker claim update is empty"));
    }

    let user_id = database::ntehelper_tracker::get_tracker_user_id(&username, &pool).await?;
    let claim = match database::ntehelper_tracker::update_tracker_claim(
        user_id, uid, nickname, region, &pool,
    )
    .await?
    {
        Ok(claim) => claim,
        Err(error) => return Ok(HttpResponse::Conflict().body(error.to_string())),
    };

    Ok(HttpResponse::Ok().json(TrackerUidClaimResponse::from(claim)))
}

#[utoipa::path(
    tag = "ntehelper/tracker",
    delete,
    path = "/api/ntehelper/tracker/uids/{uid}",
    responses(
        (status = 200, description = "Deleted tracker UID claim and stored profile", body = TrackerUidDeleteResponse),
        (status = 400, description = "Not logged in or invalid UID"),
        (status = 403, description = "Invalid origin"),
        (status = 409, description = "UID is not attached to this account"),
    )
)]
#[delete("/api/ntehelper/tracker/uids/{uid}")]
/// Deletes the authenticated owner's claim and profile, reporting the removed pull count.
async fn delete_tracker_uid_claim(
    request: HttpRequest,
    session: Session,
    uid: web::Path<String>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !super::valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Some(username) = current_username(&session) else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    let Some(uid) = validate_tracker_uid(&uid) else {
        return Ok(HttpResponse::BadRequest().body("Invalid tracker UID"));
    };

    let user_id = database::ntehelper_tracker::get_tracker_user_id(&username, &pool).await?;
    match database::ntehelper_tracker::delete_tracker_uid_profile(user_id, uid, &pool).await? {
        Ok(deleted_pulls) => {
            Ok(HttpResponse::Ok().json(TrackerUidDeleteResponse { deleted_pulls }))
        }
        Err(error) => Ok(HttpResponse::Conflict().body(error.to_string())),
    }
}

#[utoipa::path(
    tag = "ntehelper/tracker",
    get,
    path = "/api/ntehelper/tracker/{uid}",
    responses((status = 200, description = "Public tracker UID page data", body = TrackerUidPageResponse))
)]
#[get("/api/ntehelper/tracker/{uid}")]
/// Returns a public tracker profile with optional viewer-specific management annotations.
async fn get_tracker_uid(
    session: Session,
    uid: web::Path<String>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Some(uid) = validate_tracker_uid(&uid) else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let viewer_user_id = match current_username(&session) {
        Some(username) => database::ntehelper_tracker::get_tracker_user_id(&username, &pool)
            .await
            .ok(),
        None => None,
    };
    let response = tracker_response(uid, viewer_user_id, &pool).await?;

    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    tag = "ntehelper/tracker",
    post,
    path = "/api/ntehelper/tracker/{uid}/import",
    request_body = TrackerImportRequest,
    responses(
        (status = 200, description = "Imported tracker pulls", body = TrackerImportResponse),
        (status = 400, description = "Not logged in or invalid import"),
        (status = 403, description = "Invalid origin or non-owner"),
        (status = 404, description = "UID is not claimed"),
        (status = 413, description = "Import limit reached"),
    )
)]
#[post("/api/ntehelper/tracker/{uid}/import")]
/// Authorizes ownership before reading, validating, and atomically merging exported pulls.
///
/// The response distinguishes inserted rows, changed duplicates, and unchanged records;
/// body, record-count, and stored-history limits produce HTTP 413.
async fn post_tracker_import(
    request: HttpRequest,
    session: Session,
    uid: web::Path<String>,
    payload: web::Payload,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !super::valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Some(username) = current_username(&session) else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    let Some(uid) = validate_tracker_uid(&uid) else {
        return Ok(HttpResponse::BadRequest().body("Invalid tracker UID"));
    };
    let user_id = database::ntehelper_tracker::get_tracker_user_id(&username, &pool).await?;

    match owner_claim(uid, user_id, &pool).await? {
        OwnerClaim::Missing => return Ok(HttpResponse::NotFound().finish()),
        OwnerClaim::Forbidden => return Ok(HttpResponse::Forbidden().finish()),
        OwnerClaim::Allowed => {}
    }

    let body = match read_import_body(payload).await {
        Ok(body) => body,
        Err(response) => return Ok(response),
    };
    let (pulls, received) = match normalize_tracker_exports(uid, body) {
        Ok(normalized) => normalized,
        Err(error) => return Ok(error.response()),
    };

    let Some(result) = database::ntehelper_tracker::import_tracker_pulls(
        uid,
        &pulls,
        TRACKER_MAX_STORED_RECORDS,
        &pool,
    )
    .await?
    else {
        return Ok(HttpResponse::PayloadTooLarge().body("Tracker UID pull limit reached"));
    };

    let imported = result.inserted;
    let updated = result.updated;
    let skipped_duplicate = received.saturating_sub((imported + updated) as usize) as i64;

    Ok(HttpResponse::Ok().json(TrackerImportResponse {
        uid: uid.to_string(),
        received,
        imported,
        updated,
        skipped_duplicate,
        inserted: imported,
        duplicates: skipped_duplicate,
        total: result.total,
    }))
}

#[utoipa::path(
    tag = "ntehelper/tracker",
    delete,
    path = "/api/ntehelper/tracker/{uid}/pulls",
    responses(
        (status = 200, description = "Cleared tracker pulls"),
        (status = 400, description = "Not logged in or invalid UID"),
        (status = 403, description = "Invalid origin or non-owner"),
        (status = 404, description = "UID is not claimed"),
    )
)]
#[delete("/api/ntehelper/tracker/{uid}/pulls")]
/// Clears an owner's pull history while retaining the UID claim.
async fn delete_tracker_pulls(
    request: HttpRequest,
    session: Session,
    uid: web::Path<String>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !super::valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Some(username) = current_username(&session) else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    let Some(uid) = validate_tracker_uid(&uid) else {
        return Ok(HttpResponse::BadRequest().body("Invalid tracker UID"));
    };
    let user_id = database::ntehelper_tracker::get_tracker_user_id(&username, &pool).await?;

    match owner_claim(uid, user_id, &pool).await? {
        OwnerClaim::Missing => return Ok(HttpResponse::NotFound().finish()),
        OwnerClaim::Forbidden => return Ok(HttpResponse::Forbidden().finish()),
        OwnerClaim::Allowed => {}
    }

    let deleted = database::ntehelper_tracker::clear_tracker_pulls(uid, &pool).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({ "deleted": deleted })))
}

enum OwnerClaim {
    Allowed,
    Missing,
    Forbidden,
}

/// Classifies a claim for mutation authorization.
///
/// Detached claims are treated as missing; another user's attached claim is forbidden.
async fn owner_claim(uid: i64, user_id: i64, pool: &PgPool) -> ApiResult<OwnerClaim> {
    let Some(claim) = database::ntehelper_tracker::get_tracker_claim(uid, pool).await? else {
        return Ok(OwnerClaim::Missing);
    };

    Ok(if claim.owner_user_id == Some(user_id) {
        OwnerClaim::Allowed
    } else if claim.owner_user_id.is_none() {
        OwnerClaim::Missing
    } else {
        OwnerClaim::Forbidden
    })
}

/// Builds the public page only when the claim has stored history.
///
/// Missing or empty profiles return no claim or pulls. Management UI is enabled only
/// for a present viewer ID matching a present owner ID.
async fn tracker_response(
    uid: i64,
    viewer_user_id: Option<i64>,
    pool: &PgPool,
) -> ApiResult<TrackerUidPageResponse> {
    let Some(claim) = database::ntehelper_tracker::get_tracker_claim(uid, pool).await? else {
        return Ok(TrackerUidPageResponse {
            claim: None,
            pulls: Vec::new(),
            viewer_can_manage: false,
        });
    };
    if !claim.has_profile {
        return Ok(TrackerUidPageResponse {
            claim: None,
            pulls: Vec::new(),
            viewer_can_manage: false,
        });
    }
    let viewer_can_manage = viewer_owns_claim(viewer_user_id, claim.owner_user_id);
    let pulls = database::ntehelper_tracker::tracker_pulls_for_uid(uid, pool)
        .await?
        .into_iter()
        .map(TrackerPullResponse::from)
        .collect();

    Ok(TrackerUidPageResponse {
        claim: Some(TrackerUidClaimResponse::from(claim)),
        pulls,
        viewer_can_manage,
    })
}

/// Treats missing or malformed session usernames as unauthenticated.
fn current_username(session: &Session) -> Option<String> {
    session.get::<String>("username").ok().flatten()
}

/// Trims and parses only decimal UIDs with the configured NTE length and prefix.
fn validate_tracker_uid(value: &str) -> Option<i64> {
    let trimmed = value.trim();
    if trimmed.len() != TRACKER_UID_LEN
        || !trimmed.starts_with(TRACKER_UID_PREFIX)
        || !trimmed.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }

    trimmed.parse::<i64>().ok()
}

/// Returns a trimmed, supported region label; matching is case-sensitive.
fn validate_tracker_region(value: &str) -> Option<&str> {
    match value.trim() {
        "asia" | "europe" | "america" | "china" => Some(value.trim()),
        _ => None,
    }
}

/// Trims a nickname, enforcing the character cap and rejecting control characters.
fn validate_tracker_nickname(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.chars().count() > TRACKER_NICKNAME_MAX_CHARS {
        return None;
    }
    if trimmed.chars().any(char::is_control) {
        return None;
    }

    Some(trimmed)
}

/// Bounds streamed bytes before decoding JSON.
///
/// Malformed chunks/JSON return HTTP 400; crossing the byte cap returns HTTP 413.
async fn read_import_body(mut payload: web::Payload) -> Result<TrackerImportRequest, HttpResponse> {
    let mut bytes = web::BytesMut::new();

    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|_| HttpResponse::BadRequest().body("Invalid request body"))?;
        if bytes.len() + chunk.len() > TRACKER_IMPORT_MAX_BODY_BYTES {
            return Err(HttpResponse::PayloadTooLarge().body("Tracker import body is too large"));
        }
        bytes.extend_from_slice(&chunk);
    }

    serde_json::from_slice::<TrackerImportRequest>(&bytes)
        .map_err(|_| HttpResponse::BadRequest().body("Invalid tracker import JSON"))
}

/// Validates export versions, UID/pool consistency, field shapes, and input limits.
///
/// Returns unique record IDs plus the original received count. Later duplicates replace
/// earlier records; the resulting vector has no guaranteed order because persistence
/// uses record IDs and explicit timestamps.
fn normalize_tracker_exports(
    uid: i64,
    body: TrackerImportRequest,
) -> Result<(Vec<database::ntehelper_tracker::DbTrackerPull>, usize), TrackerImportError> {
    if body.exports.is_empty() {
        return Err(TrackerImportError::bad_request(
            "Import requires at least one export",
        ));
    }
    if body.exports.len() > TRACKER_IMPORT_MAX_EXPORTS {
        return Err(TrackerImportError::too_large(
            "Too many exports in one import",
        ));
    }

    let received = body
        .exports
        .iter()
        .map(|export| export.records.len())
        .sum::<usize>();
    if received > TRACKER_IMPORT_MAX_RECORDS {
        return Err(TrackerImportError::too_large(
            "Too many records in one import",
        ));
    }

    let mut records_by_uid = HashMap::new();

    for export in body.exports {
        if export.format != "nte-history-export" || export.format_version != 1 {
            return Err(TrackerImportError::bad_request(
                "Unsupported tracker export format",
            ));
        }
        if validate_tracker_uid(&export.user_uid) != Some(uid) {
            return Err(TrackerImportError::bad_request(
                "Import UID does not match target tracker UID",
            ));
        }

        banner_type_for_pool(&export.banner.id)
            .ok_or_else(|| TrackerImportError::bad_request("Unsupported tracker banner"))?;

        for record in export.records {
            if record.pool_group_id != export.banner.id {
                return Err(TrackerImportError::bad_request(
                    "Record pool does not match export banner",
                ));
            }

            let record_uid =
                safe_identifier(&record.uid, TRACKER_RECORD_UID_MAX_CHARS, "record UID")?;
            let reward_id = safe_text(
                &record.reward_id,
                TRACKER_REWARD_TEXT_MAX_CHARS,
                "reward ID",
            )?;
            let result_type = record
                .result_type
                .map(|value| safe_text(&value, TRACKER_TEXT_MAX_CHARS, "result type"))
                .transpose()?;

            if !valid_timestamp(&record.timestamp) {
                return Err(TrackerImportError::bad_request("Invalid tracker timestamp"));
            }
            if record.timestamp_group_ordinal < 0 {
                return Err(TrackerImportError::bad_request(
                    "Invalid timestamp group ordinal",
                ));
            }
            if record.quantity.is_some_and(|value| value < 0) {
                return Err(TrackerImportError::bad_request("Invalid quantity"));
            }

            records_by_uid.insert(
                record_uid.clone(),
                database::ntehelper_tracker::DbTrackerPull {
                    uid,
                    record_uid,
                    pool_group_id: export.banner.id.clone(),
                    timestamp_raw: record.timestamp,
                    timestamp_group_ordinal: Some(record.timestamp_group_ordinal),
                    roll_result: record.roll_result,
                    result_type,
                    reward_id,
                    quantity: record.quantity,
                    imported_at: Utc::now(),
                },
            );
        }
    }

    Ok((records_by_uid.into_values().collect(), received))
}

/// Maps recognized external pool-group IDs to API labels; unknown pools return `None`.
fn banner_type_for_pool(pool_group_id: &str) -> Option<&'static str> {
    match pool_group_id {
        "Lottery_LimitedCharacter" => Some("limited-character"),
        "Lottery_Permanent" => Some("permanent-character"),
        "Arc_MiracleBox" => Some("arc"),
        _ => None,
    }
}

/// Accepts nonempty bounded ASCII identifiers containing letters, digits, underscores, or hyphens.
fn safe_identifier(
    value: &str,
    max_chars: usize,
    label: &'static str,
) -> Result<String, TrackerImportError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > max_chars
        || !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(TrackerImportError::bad_request(format!("Invalid {label}")));
    }

    Ok(trimmed.to_string())
}

/// Accepts bounded import tokens, additionally permitting periods and colons.
///
/// Despite the name, this is not a free-text validator; whitespace/control characters
/// and other punctuation are rejected after trimming.
fn safe_text(
    value: &str,
    max_chars: usize,
    label: &'static str,
) -> Result<String, TrackerImportError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > max_chars
        || !trimmed
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':'))
    {
        return Err(TrackerImportError::bad_request(format!("Invalid {label}")));
    }

    Ok(trimmed.to_string())
}

/// Checks the fixed `YYYY-MM-DD HH:MM:SS` shape without calendar validation.
///
/// Raw timestamps are retained as text; this does not parse a timezone or date.
fn valid_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 19
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
        && bytes[10] == b' '
        && bytes[11..13].iter().all(u8::is_ascii_digit)
        && bytes[13] == b':'
        && bytes[14..16].iter().all(u8::is_ascii_digit)
        && bytes[16] == b':'
        && bytes[17..19].iter().all(u8::is_ascii_digit)
}

#[derive(Debug)]
struct TrackerImportError {
    status: StatusCode,
    message: String,
}

impl TrackerImportError {
    /// Creates a validation failure that becomes HTTP 400.
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    /// Creates a configured-limit failure that becomes HTTP 413.
    fn too_large(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            message: message.into(),
        }
    }

    /// Converts the validation error into its status and plain-text message.
    fn response(self) -> HttpResponse {
        HttpResponse::build(self.status).body(self.message)
    }
}

impl From<database::ntehelper_tracker::DbTrackerUidClaim> for TrackerUidClaimResponse {
    fn from(claim: database::ntehelper_tracker::DbTrackerUidClaim) -> Self {
        TrackerUidClaimResponse {
            uid: claim.uid.to_string(),
            nickname: claim.nickname,
            region: claim.region,
            owner_username: claim.owner_username,
            claim_source: claim.claim_source,
            has_profile: claim.has_profile,
            claimed_at: claim.claimed_at,
            updated_at: claim.updated_at,
        }
    }
}

impl From<database::ntehelper_tracker::DbTrackerPull> for TrackerPullResponse {
    fn from(pull: database::ntehelper_tracker::DbTrackerPull) -> Self {
        TrackerPullResponse {
            uid: pull.uid.to_string(),
            record_uid: pull.record_uid,
            pool_group_id: pull.pool_group_id,
            timestamp_raw: pull.timestamp_raw,
            timestamp_group_ordinal: pull.timestamp_group_ordinal.unwrap_or_default(),
            reward_id: pull.reward_id,
            roll_result: pull.roll_result,
            result_type: pull.result_type,
            quantity: pull.quantity,
            source_type: Some("exporter".to_string()),
            imported_at: pull.imported_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_export(pool_group_id: &str, rank: Option<&str>) -> RawTrackerExport {
        RawTrackerExport {
            format: "nte-history-export".to_string(),
            format_version: 1,
            user_uid: "211234567890".to_string(),
            banner: RawTrackerBanner {
                id: pool_group_id.to_string(),
            },
            records: vec![RawTrackerRecord {
                uid: "a940530e488a0e537af6070043fbc19a".to_string(),
                pool_group_id: pool_group_id.to_string(),
                timestamp: "2026-06-07 19:28:21".to_string(),
                timestamp_group_ordinal: 0,
                roll_result: Some(5),
                result_type: Some("dice".to_string()),
                _reward_type: Some("arc".to_string()),
                reward_id: "fork_vine".to_string(),
                _reward_name: Some("Be Happy".to_string()),
                _reward_rank: rank.map(str::to_string),
                quantity: Some(1),
            }],
        }
    }

    #[test]
    fn validates_numeric_tracker_uids() {
        assert_eq!(validate_tracker_uid("211234567890"), Some(211234567890));
        assert_eq!(validate_tracker_uid("abc"), None);
        assert_eq!(validate_tracker_uid("221234567890"), None);
        assert_eq!(validate_tracker_uid("21123456789"), None);
    }

    #[test]
    fn normalizes_known_export_pools_and_ranks() {
        let uid = 211234567890;
        let body = TrackerImportRequest {
            exports: vec![
                sample_export("Lottery_LimitedCharacter", Some("S")),
                sample_export("Lottery_Permanent", Some("A")),
                sample_export("Arc_MiracleBox", Some("B")),
            ],
        };

        let (pulls, received) = normalize_tracker_exports(uid, body).unwrap();

        assert_eq!(received, 3);
        assert_eq!(pulls.len(), 1);
        let pull = pulls.first().unwrap();
        assert_eq!(pull.uid, uid);
        assert_eq!(pull.pool_group_id, "Arc_MiracleBox");
    }

    #[test]
    fn ignores_stale_export_metadata_for_unknown_reward_ids() {
        let uid = 211234567890;
        let body = TrackerImportRequest {
            exports: vec![RawTrackerExport {
                format: "nte-history-export".to_string(),
                format_version: 1,
                user_uid: "211234567890".to_string(),
                banner: RawTrackerBanner {
                    id: "Lottery_LimitedCharacter".to_string(),
                },
                records: vec![RawTrackerRecord {
                    uid: "unknownrewardrecord".to_string(),
                    pool_group_id: "Lottery_LimitedCharacter".to_string(),
                    timestamp: "2026-07-08 07:12:37".to_string(),
                    timestamp_group_ordinal: 3,
                    roll_result: Some(1),
                    result_type: Some("dice".to_string()),
                    _reward_type: Some("item".to_string()),
                    reward_id: "mystery_reward".to_string(),
                    _reward_name: Some("Mystery Reward".to_string()),
                    _reward_rank: Some("A".to_string()),
                    quantity: Some(1),
                }],
            }],
        };

        let (pulls, received) = normalize_tracker_exports(uid, body).unwrap();

        assert_eq!(received, 1);
        assert_eq!(pulls.len(), 1);
        let pull = pulls.first().unwrap();
        assert_eq!(pull.reward_id, "mystery_reward");
        assert_eq!(pull.timestamp_raw, "2026-07-08 07:12:37");
        assert_eq!(pull.roll_result, Some(1));
        assert_eq!(pull.result_type, Some("dice".to_string()));
        assert_eq!(pull.quantity, Some(1));
    }

    #[test]
    fn rejects_malformed_tracker_records() {
        let uid = 211234567890;
        let mut export = sample_export("Lottery_LimitedCharacter", Some("S"));
        export.records[0].timestamp = "2026/06/07".to_string();

        assert!(normalize_tracker_exports(
            uid,
            TrackerImportRequest {
                exports: vec![export],
            },
        )
        .is_err());
    }

    #[test]
    fn rejects_mismatched_export_user_uid() {
        let mut export = sample_export("Lottery_LimitedCharacter", Some("S"));
        export.user_uid = "211234567891".to_string();

        let error = match normalize_tracker_exports(
            211234567890,
            TrackerImportRequest {
                exports: vec![export],
            },
        ) {
            Ok(_) => panic!("expected mismatched user_uid import to fail"),
            Err(error) => error,
        };

        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(
            error.message,
            "Import UID does not match target tracker UID"
        );
    }

    #[test]
    fn tracker_claim_detach_migration_allows_multiple_self_claims() {
        let migration = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/migrations/20260616010000_ntehelper_tracker_claim_detach.sql"
        ));

        assert!(migration.contains("CREATE INDEX ntehelper_tracker_uid_claim_self_owner_idx"));
        assert!(
            !migration.contains("CREATE UNIQUE INDEX ntehelper_tracker_uid_claim_self_owner_idx")
        );
    }
}

/// Detached claims have no owner; anonymous viewers must not be offered management UI.
fn viewer_owns_claim(viewer: Option<i64>, owner: Option<i64>) -> bool {
    viewer.is_some_and(|viewer| owner == Some(viewer))
}

#[cfg(test)]
mod ownership_tests {
    use super::viewer_owns_claim;

    #[test]
    fn detached_claims_are_never_manageable() {
        assert!(!viewer_owns_claim(None, None));
        assert!(!viewer_owns_claim(Some(1), None));
        assert!(!viewer_owns_claim(None, Some(1)));
        assert!(!viewer_owns_claim(Some(2), Some(1)));
        assert!(viewer_owns_claim(Some(1), Some(1)));
    }
}
