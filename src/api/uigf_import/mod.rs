//! UIGF multi-game authorization, ordered history cutoffs and atomic unofficial persistence.

use std::collections::HashMap;

use crate::gacha::imports::{NormalizedPull, PullItem, PullPool, PullProvenance};
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::ApiResult,
    database, {GachaType, GiGachaType, ZzzGachaType},
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "uigf-import")),
    paths(post_uigf_import),
    components(schemas(UigfImportParams, UigfImportSummary, UigfGameImportSummary)),
)]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module's routes and any shared application data.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_uigf_import);
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
struct UigfImportParams {
    data: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct UigfImportSummary {
    hsr: UigfGameImportSummary,
    gi: UigfGameImportSummary,
    zzz: UigfGameImportSummary,
}

#[derive(Default, serde::Serialize, utoipa::ToSchema)]
struct UigfGameImportSummary {
    imported: u64,
    unchanged: u64,
    repaired: u64,
    skipped_uids: u64,
    skipped_records: u64,
}

#[derive(serde::Deserialize, Clone)]
#[serde(untagged)]
enum StringOrInt {
    String(String),
    Int(i32),
}

impl StringOrInt {
    /// Parses a string UID or returns an already-numeric UID without range validation.
    fn parse(self) -> Result<i32, std::num::ParseIntError> {
        match self {
            StringOrInt::String(s) => s.parse(),
            StringOrInt::Int(i) => Ok(i),
        }
    }
}

#[derive(serde::Deserialize)]
struct Uigf {
    info: UigfInfo,
    #[serde(default)]
    hk4e: Vec<UigfHk4e>,
    #[serde(default)]
    hkrpg: Vec<UigfHkrpg>,
    #[serde(default)]
    nap: Vec<UigfNap>,
}

#[derive(serde::Deserialize)]
struct UigfInfo {
    version: String,
}

#[derive(serde::Deserialize)]
struct UigfHk4e {
    uid: StringOrInt,
    timezone: i32,
    list: Vec<UigfHk4eEntry>,
}

#[derive(serde::Deserialize)]
struct UigfHk4eEntry {
    id: String,
    uigf_gacha_type: String,
    item_id: String,
    item_type: String,
    time: String,
}

#[derive(serde::Deserialize)]
struct UigfHkrpg {
    uid: StringOrInt,
    timezone: i32,
    list: Vec<UigfHkrpgEntry>,
}

#[derive(serde::Deserialize)]
struct UigfHkrpgEntry {
    id: String,
    gacha_type: String,
    item_id: String,
    item_type: String,
    time: String,
}

#[derive(serde::Deserialize)]
struct UigfNap {
    uid: StringOrInt,
    timezone: i32,
    list: Vec<UigfNapEntry>,
}

#[derive(serde::Deserialize)]
struct UigfNapEntry {
    id: String,
    gacha_type: String,
    item_id: String,
    item_type: String,
    time: String,
}

/// Parses a `vMAJOR.MINOR` version, returning None for any other syntax.
fn parse_uigf_version(version: &str) -> Option<(u32, u32)> {
    let version = version.strip_prefix('v')?;
    let (major, minor) = version.split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

/// Allows an admin or a verified HSR connection; database failures propagate.
async fn check_hsr_auth(admin: bool, username: &str, uid: i32, pool: &PgPool) -> ApiResult<bool> {
    let allowed = admin
        || database::connections::get_by_username(username, pool)
            .await?
            .iter()
            .find(|c| c.uid == uid)
            .map(|c| c.verified)
            .unwrap_or_default();
    Ok(allowed)
}

/// Allows an admin or a verified ZZZ connection; database failures propagate.
async fn check_zzz_auth(admin: bool, username: &str, uid: i32, pool: &PgPool) -> ApiResult<bool> {
    let allowed = admin
        || database::zzz::connections::get_by_username(username, pool)
            .await?
            .iter()
            .find(|c| c.uid == uid)
            .map(|c| c.verified)
            .unwrap_or_default();
    Ok(allowed)
}

/// Allows an admin or a verified GI connection; database failures propagate.
async fn check_gi_auth(admin: bool, username: &str, uid: i32, pool: &PgPool) -> ApiResult<bool> {
    let allowed = admin
        || database::gi::connections::get_by_username(username, pool)
            .await?
            .iter()
            .find(|c| c.uid == uid)
            .map(|c| c.verified)
            .unwrap_or_default();
    Ok(allowed)
}

#[utoipa::path(
    tag = "uigf-import",
    post,
    path = "/api/uigf-import",
    request_body = UigfImportParams,
    responses(
        (status = 200, description = "UIGF imported", body = UigfImportSummary),
        (status = 400, description = "Invalid data or version"),
        (status = 403, description = "Not authorized"),
    )
)]
#[post("/api/uigf-import")]
/// Imports authorized UIGF v4-or-later profiles in one cross-game transaction.
/// Unauthorized UIDs increment skip counters; non-admin history stops at each pool's
/// first overlap. Malformed data or batches return 400 before pulls or stats are written.
async fn post_uigf_import(
    session: Session,
    params: web::Json<UigfImportParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let Ok(uigf) = serde_json::from_str::<Uigf>(&params.data) else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    match parse_uigf_version(&uigf.info.version) {
        Some((major, _minor)) if major >= 4 => {}
        _ => return Ok(HttpResponse::BadRequest().finish()),
    }

    let admin = database::admins::exists(&username, &pool).await?;
    let mut normalized_pulls = Vec::new();
    let mut hsr_skipped = UigfGameImportSummary::default();
    let mut gi_skipped = UigfGameImportSummary::default();
    let mut zzz_skipped = UigfGameImportSummary::default();

    for entry in &uigf.hkrpg {
        let Ok(uid) = entry.uid.clone().parse() else {
            return Ok(HttpResponse::BadRequest().finish());
        };
        if !check_hsr_auth(admin, &username, uid, &pool).await? {
            hsr_skipped.skipped_uids += 1;
            hsr_skipped.skipped_records += entry.list.len() as u64;
            continue;
        }
        let mut by_pool: HashMap<GachaType, Vec<NormalizedPull>> = HashMap::new();
        for pull in &entry.list {
            let gacha_type = match pull.gacha_type.as_str() {
                "1" => GachaType::Standard,
                "2" => GachaType::Departure,
                "11" => GachaType::Special,
                "12" => GachaType::Lc,
                "21" => GachaType::Collab,
                "22" => GachaType::CollabLc,
                _ => return Ok(HttpResponse::BadRequest().finish()),
            };
            let Ok(pull) = parse_pull(
                uid,
                PullPool::Hsr(gacha_type),
                &pull.id,
                &pull.item_id,
                &pull.item_type,
                &pull.time,
                entry.timezone,
            ) else {
                return Ok(HttpResponse::BadRequest().finish());
            };
            by_pool.entry(gacha_type).or_default().push(pull);
        }
        for (gacha_type, pulls) in by_pool {
            let earliest =
                database::warps::get_earliest_timestamp_by_uid_by_pool(gacha_type, uid, &pool)
                    .await?;
            normalized_pulls.extend(before_cutoff(pulls, earliest, admin));
        }
    }

    for entry in &uigf.nap {
        let Ok(uid) = entry.uid.clone().parse() else {
            return Ok(HttpResponse::BadRequest().finish());
        };
        if !check_zzz_auth(admin, &username, uid, &pool).await? {
            zzz_skipped.skipped_uids += 1;
            zzz_skipped.skipped_records += entry.list.len() as u64;
            continue;
        }
        let mut by_pool: HashMap<ZzzGachaType, Vec<NormalizedPull>> = HashMap::new();
        for pull in &entry.list {
            let gacha_type = match ZzzGachaType::from_uigf_id(&pull.gacha_type) {
                Some(pool) => pool,
                None => return Ok(HttpResponse::BadRequest().finish()),
            };
            let Ok(pull) = parse_pull(
                uid,
                PullPool::Zzz(gacha_type),
                &pull.id,
                &pull.item_id,
                &pull.item_type,
                &pull.time,
                entry.timezone,
            ) else {
                return Ok(HttpResponse::BadRequest().finish());
            };
            by_pool.entry(gacha_type).or_default().push(pull);
        }
        for (gacha_type, pulls) in by_pool {
            let earliest = database::zzz::signals::get_earliest_timestamp_by_uid_by_pool(
                gacha_type, uid, &pool,
            )
            .await?;
            normalized_pulls.extend(before_cutoff(pulls, earliest, admin));
        }
    }

    for entry in &uigf.hk4e {
        let Ok(uid) = entry.uid.clone().parse() else {
            return Ok(HttpResponse::BadRequest().finish());
        };
        if !check_gi_auth(admin, &username, uid, &pool).await? {
            gi_skipped.skipped_uids += 1;
            gi_skipped.skipped_records += entry.list.len() as u64;
            continue;
        }
        let mut by_pool: HashMap<GiGachaType, Vec<NormalizedPull>> = HashMap::new();
        for pull in &entry.list {
            // UIGF's normalized field combines raw character banner types 301
            // and 400 into 301; 400 belongs to gacha_type, not uigf_gacha_type.
            let gacha_type = match pull.uigf_gacha_type.as_str() {
                "100" => GiGachaType::Beginner,
                "200" => GiGachaType::Standard,
                "301" => GiGachaType::Character,
                "302" => GiGachaType::Weapon,
                "500" => GiGachaType::Chronicled,
                _ => return Ok(HttpResponse::BadRequest().finish()),
            };
            let Ok(pull) = parse_pull(
                uid,
                PullPool::Gi(gacha_type),
                &pull.id,
                &pull.item_id,
                &pull.item_type,
                &pull.time,
                entry.timezone,
            ) else {
                return Ok(HttpResponse::BadRequest().finish());
            };
            by_pool.entry(gacha_type).or_default().push(pull);
        }
        for (gacha_type, pulls) in by_pool {
            let earliest =
                database::gi::wishes::get_earliest_timestamp_by_uid_by_pool(gacha_type, uid, &pool)
                    .await?;
            normalized_pulls.extend(before_cutoff(pulls, earliest, admin));
        }
    }

    let Ok(batch) =
        crate::gacha::imports::ImportBatch::new(normalized_pulls, PullProvenance::Unofficial)
    else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    let mut hsr = hsr_skipped;
    let mut gi = gi_skipped;
    let mut zzz = zzz_skipped;
    for pull in batch.pulls() {
        match pull.pool {
            crate::gacha::imports::PullPool::Hsr(_) => hsr.unchanged += 1,
            crate::gacha::imports::PullPool::Gi(_) => gi.unchanged += 1,
            crate::gacha::imports::PullPool::Zzz(_) => zzz.unchanged += 1,
        }
    }
    let persisted = crate::gacha::imports::persist_batch_in_transaction(&batch, &pool).await?;
    hsr.imported = persisted.hsr_changed;
    gi.imported = persisted.gi_changed;
    zzz.imported = persisted.zzz_changed;
    hsr.unchanged -= hsr.imported;
    gi.unchanged -= gi.imported;
    zzz.unchanged -= zzz.imported;

    Ok(HttpResponse::Ok().json(UigfImportSummary { hsr, gi, zzz }))
}

/// Yields records before the first overlap in source order, discarding the whole
/// remaining pool tail even when later timestamps are older. Admins bypass this
/// cutoff; pool-item validation applies later to the retained batch.
fn before_cutoff(
    pulls: Vec<NormalizedPull>,
    earliest: Option<DateTime<Utc>>,
    admin: bool,
) -> impl Iterator<Item = NormalizedPull> {
    pulls
        .into_iter()
        .take_while(move |pull| admin || earliest.is_none_or(|time| pull.timestamp < time))
}
/// Parses a UIGF record's numeric IDs, localized item type and fixed-offset time.
/// Returns an unofficial typed pull without I/O; pool compatibility is validated only
/// after cutoff filtering, while malformed fields return parsing errors.
fn parse_pull(
    uid: i32,
    pool: PullPool,
    id: &str,
    item_id: &str,
    item_type: &str,
    time: &str,
    timezone: i32,
) -> anyhow::Result<NormalizedPull> {
    let item_id = item_id.parse()?;
    let item = match (pool, item_type) {
        (PullPool::Hsr(_), "Character" | "角色")
        | (PullPool::Gi(_), "Character" | "角色")
        | (PullPool::Zzz(_), "Character" | "Agents" | "代理人") => PullItem::Character(item_id),
        (PullPool::Hsr(_), "Light Cone" | "光锥" | "光錐") => PullItem::LightCone(item_id),
        (PullPool::Gi(_), "Weapon" | "Weapons" | "武器") => PullItem::Weapon(item_id),
        (PullPool::Zzz(_), "W-Engines" | "音擎") => PullItem::WEngine(item_id),
        (PullPool::Zzz(_), "Bangboo" | "邦布") => PullItem::Bangboo(item_id),
        _ => anyhow::bail!("invalid item type"),
    };
    let tz = timezone
        .checked_mul(3600)
        .and_then(FixedOffset::east_opt)
        .ok_or_else(|| anyhow::anyhow!("invalid timezone"))?;
    let timestamp = NaiveDateTime::parse_from_str(time, "%Y-%m-%d %H:%M:%S")?
        .and_local_timezone(tz)
        .single()
        .ok_or_else(|| anyhow::anyhow!("invalid local time"))?
        .to_utc();
    let pull = NormalizedPull {
        uid,
        id: id.parse()?,
        pool,
        item,
        timestamp,
        provenance: PullProvenance::Unofficial,
    };
    Ok(pull)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cutoff_stops_at_first_overlap_even_for_unordered_input() {
        let make = |id: &str, time| {
            parse_pull(
                1,
                PullPool::Hsr(GachaType::Standard),
                id,
                "1",
                "Character",
                time,
                0,
            )
            .unwrap()
        };
        let pulls = vec![
            make("1", "2024-01-01 00:00:00"),
            make("2", "2024-01-03 00:00:00"),
            make("3", "2024-01-01 00:00:00"),
        ];
        let cutoff = make("0", "2024-01-02 00:00:00").timestamp;
        assert_eq!(before_cutoff(pulls.clone(), Some(cutoff), false).count(), 1);
        assert_eq!(before_cutoff(pulls, Some(cutoff), true).count(), 3);
    }
    #[test]
    fn malformed_pull_is_rejected() {
        for (item_type, timezone) in [("wrong", 0), ("Character", i32::MAX)] {
            assert!(parse_pull(
                1,
                PullPool::Hsr(GachaType::Standard),
                "1",
                "1",
                item_type,
                "2024-01-01 00:00:00",
                timezone
            )
            .is_err());
        }
        let invalid_pool_item = parse_pull(
            1,
            PullPool::Zzz(ZzzGachaType::Bangboo),
            "1",
            "1",
            "Character",
            "2024-01-01 00:00:00",
            0,
        )
        .unwrap();
        assert!(crate::gacha::imports::ImportBatch::new(
            [invalid_pool_item.clone()],
            PullProvenance::Unofficial
        )
        .is_err());
        // Pool identity validation applies to retained records. An overlapping
        // record and its tail are discarded before the single batch validation.
        let retained: Vec<_> = before_cutoff(
            vec![invalid_pool_item.clone()],
            Some(invalid_pool_item.timestamp),
            false,
        )
        .collect();
        assert!(
            crate::gacha::imports::ImportBatch::new(retained, PullProvenance::Unofficial).is_ok()
        );
    }
}

#[cfg(test)]
mod endpoint_tests {
    use super::*;
    use actix_session::SessionExt;
    use actix_web::{dev::Service, test, App};

    #[actix_web::test]
    async fn malformed_item_is_400_and_unauthorized_uid_is_counted() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL required"))
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        let suffix = uuid::Uuid::new_v4().simple().to_string();
        let username = format!("u{}", &suffix[..20]);
        let uid = 1_400_000_000 + (uuid::Uuid::new_v4().as_u128() % 100_000_000) as i32;
        sqlx::query("INSERT INTO users (username, password) VALUES ($1, '')")
            .bind(&username)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO mihomo (uid, region, name, level, signature, avatar_icon, achievement_count) VALUES ($1, 'na', '', 1, '', '', 0)").bind(uid).execute(&pool).await.unwrap();
        database::connections::set(
            &database::connections::DbConnection {
                uid,
                username: username.clone(),
                verified: true,
                private: false,
            },
            &pool,
        )
        .await
        .unwrap();
        let session_username = username.clone();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .wrap_fn(move |request, service| {
                    request
                        .get_session()
                        .insert("username", &session_username)
                        .unwrap();
                    service.call(request)
                })
                .service(post_uigf_import),
        )
        .await;
        let record = serde_json::json!({"id":"1","gacha_type":"1","item_id":"1","item_type":"malformed","time":"2024-01-01 00:00:00"});
        let data = serde_json::json!({"info":{"version":"v4.1"},"hkrpg":[{"uid":uid,"timezone":0,"list":[record.clone()]}]});
        let request = test::TestRequest::post()
            .uri("/api/uigf-import")
            .set_json(serde_json::json!({"data":data.to_string()}))
            .to_request();
        assert_eq!(
            test::call_service(&app, request).await.status(),
            actix_web::http::StatusCode::BAD_REQUEST
        );
        let data = serde_json::json!({"info":{"version":"v4.1"},"hkrpg":[{"uid":uid+1,"timezone":0,"list":[record.clone(),record]}]});
        let request = test::TestRequest::post()
            .uri("/api/uigf-import")
            .set_json(serde_json::json!({"data":data.to_string()}))
            .to_request();
        let response = test::call_service(&app, request).await;
        assert_eq!(response.status(), actix_web::http::StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(body["hsr"]["skipped_uids"], 1);
        assert_eq!(body["hsr"]["skipped_records"], 2);
        assert_eq!(body["hsr"]["imported"], 0);
        sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM mihomo WHERE uid = $1")
            .bind(uid)
            .execute(&pool)
            .await
            .unwrap();
    }
}
