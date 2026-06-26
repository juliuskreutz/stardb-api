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

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

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
    owner_username: Option<String>,
    claim_source: String,
    has_profile: bool,
    claimed_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct TrackerClaimUpdateRequest {
    nickname: String,
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
    banner_type: String,
    timestamp_raw: String,
    timestamp_group_ordinal: i32,
    reward_type: String,
    reward_id: String,
    reward_name: String,
    reward_rank: Option<String>,
    star_rank: Option<i32>,
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
    reward_type: String,
    reward_id: String,
    reward_name: String,
    reward_rank: Option<String>,
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
    let Some(nickname) = validate_tracker_nickname(&body.nickname) else {
        return Ok(HttpResponse::BadRequest().body("Invalid tracker nickname"));
    };

    let user_id = database::ntehelper_tracker::get_tracker_user_id(&username, &pool).await?;
    let claim = match database::ntehelper_tracker::update_tracker_claim_nickname(
        user_id,
        uid,
        nickname,
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
        Ok(deleted_pulls) => Ok(HttpResponse::Ok().json(TrackerUidDeleteResponse {
            deleted_pulls,
        })),
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

    let Some(result) =
        database::ntehelper_tracker::import_tracker_pulls(
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
    let skipped_duplicate = received.saturating_sub(imported as usize) as i64;

    Ok(HttpResponse::Ok().json(TrackerImportResponse {
        uid: uid.to_string(),
        received,
        imported,
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
    let viewer_can_manage = viewer_user_id == claim.owner_user_id;
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

fn current_username(session: &Session) -> Option<String> {
    session.get::<String>("username").ok().flatten()
}

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

        let banner_type = banner_type_for_pool(&export.banner.id)
            .ok_or_else(|| TrackerImportError::bad_request("Unsupported tracker banner"))?;

        for record in export.records {
            if record.pool_group_id != export.banner.id {
                return Err(TrackerImportError::bad_request(
                    "Record pool does not match export banner",
                ));
            }

            let record_uid =
                safe_identifier(&record.uid, TRACKER_RECORD_UID_MAX_CHARS, "record UID")?;
            let reward_type =
                safe_text(&record.reward_type, TRACKER_TEXT_MAX_CHARS, "reward type")?;
            let reward_id = safe_text(
                &record.reward_id,
                TRACKER_REWARD_TEXT_MAX_CHARS,
                "reward ID",
            )?;
            let reward_name = safe_display_text(
                &record.reward_name,
                TRACKER_REWARD_TEXT_MAX_CHARS,
                "reward name",
            )?;
            let result_type = record
                .result_type
                .map(|value| safe_text(&value, TRACKER_TEXT_MAX_CHARS, "result type"))
                .transpose()?;
            let reward_rank = record
                .reward_rank
                .map(|rank| normalize_rank(&rank))
                .transpose()?;
            let star_rank = star_rank_for_reward_rank(reward_rank.as_deref());

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
                    banner_type: banner_type.to_string(),
                    timestamp_raw: record.timestamp,
                    timestamp_group_ordinal: Some(record.timestamp_group_ordinal),
                    roll_result: record.roll_result,
                    result_type,
                    reward_type,
                    reward_id,
                    reward_name,
                    reward_rank,
                    star_rank,
                    quantity: record.quantity,
                    imported_at: Utc::now(),
                },
            );
        }
    }

    Ok((records_by_uid.into_values().collect(), received))
}

fn banner_type_for_pool(pool_group_id: &str) -> Option<&'static str> {
    match pool_group_id {
        "Lottery_LimitedCharacter" => Some("limited-character"),
        "Lottery_Permanent" => Some("permanent-character"),
        "Arc_MiracleBox" => Some("arc"),
        _ => None,
    }
}

fn star_rank_for_reward_rank(rank: Option<&str>) -> Option<i32> {
    match rank {
        Some("S") => Some(5),
        Some("A") => Some(4),
        Some("B") => Some(3),
        _ => None,
    }
}

fn normalize_rank(rank: &str) -> Result<String, TrackerImportError> {
    match rank.trim() {
        "S" | "A" | "B" => Ok(rank.trim().to_string()),
        _ => Err(TrackerImportError::bad_request("Unsupported reward rank")),
    }
}

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

fn safe_display_text(
    value: &str,
    max_chars: usize,
    label: &'static str,
) -> Result<String, TrackerImportError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed.chars().count() > max_chars
        || trimmed.chars().any(char::is_control)
    {
        return Err(TrackerImportError::bad_request(format!("Invalid {label}")));
    }

    Ok(trimmed.to_string())
}

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
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn too_large(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::PAYLOAD_TOO_LARGE,
            message: message.into(),
        }
    }

    fn response(self) -> HttpResponse {
        HttpResponse::build(self.status).body(self.message)
    }
}

impl From<database::ntehelper_tracker::DbTrackerUidClaim> for TrackerUidClaimResponse {
    fn from(claim: database::ntehelper_tracker::DbTrackerUidClaim) -> Self {
        TrackerUidClaimResponse {
            uid: claim.uid.to_string(),
            nickname: claim.nickname,
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
            banner_type: pull.banner_type,
            timestamp_raw: pull.timestamp_raw,
            timestamp_group_ordinal: pull.timestamp_group_ordinal.unwrap_or_default(),
            reward_type: pull.reward_type,
            reward_id: pull.reward_id,
            reward_name: pull.reward_name,
            reward_rank: pull.reward_rank,
            star_rank: pull.star_rank,
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
                reward_type: "arc".to_string(),
                reward_id: "fork_vine".to_string(),
                reward_name: "Be Happy".to_string(),
                reward_rank: rank.map(str::to_string),
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
                sample_export("Arc_MiracleBox", None),
            ],
        };

        let (pulls, received) = normalize_tracker_exports(uid, body).unwrap();

        assert_eq!(received, 3);
        assert_eq!(pulls.len(), 1);
        let pull = pulls.first().unwrap();
        assert_eq!(pull.uid, uid);
        assert_eq!(pull.star_rank, None);
        assert_eq!(pull.banner_type, "arc");
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
        assert_eq!(error.message, "Import UID does not match target tracker UID");
    }

    #[test]
    fn tracker_claim_detach_migration_allows_multiple_self_claims() {
        let migration = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/migrations/20260616010000_ntehelper_tracker_claim_detach.sql"
        ));

        assert!(migration.contains("CREATE INDEX ntehelper_tracker_uid_claim_self_owner_idx"));
        assert!(!migration.contains("CREATE UNIQUE INDEX ntehelper_tracker_uid_claim_self_owner_idx"));
    }
}
