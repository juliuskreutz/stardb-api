use std::collections::HashMap;

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

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

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

fn parse_uigf_version(version: &str) -> Option<(u32, u32)> {
    let version = version.strip_prefix('v')?;
    let (major, minor) = version.split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}

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
async fn post_uigf_import(
    session: Session,
    params: web::Json<UigfImportParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let uigf: Uigf = serde_json::from_str(&params.data)?;

    match parse_uigf_version(&uigf.info.version) {
        Some((major, _minor)) if major >= 4 => {}
        _ => return Ok(HttpResponse::BadRequest().finish()),
    }

    let admin = database::admins::exists(&username, &pool).await?;
    let mut normalized_pulls = Vec::new();
    let mut hsr_skipped = UigfGameImportSummary::default();
    let mut gi_skipped = UigfGameImportSummary::default();
    let mut zzz_skipped = UigfGameImportSummary::default();

    // HSR (hkrpg)
    for entry in &uigf.hkrpg {
        let uid = entry.uid.clone().parse()?;
        if !check_hsr_auth(admin, &username, uid, &pool).await? {
            hsr_skipped.skipped_uids += 1;
            hsr_skipped.skipped_records += entry.list.len() as u64;
            continue;
        }

        let tz = FixedOffset::east_opt(3600 * entry.timezone).unwrap();

        let mut warps_map: HashMap<GachaType, Vec<(i64, Option<i32>, Option<i32>, DateTime<Utc>)>> =
            HashMap::new();

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

            let time = NaiveDateTime::parse_from_str(&pull.time, "%Y-%m-%d %H:%M:%S")?
                .and_local_timezone(tz)
                .unwrap()
                .to_utc();

            let id: i64 = pull.id.parse()?;
            let item_id: i32 = pull.item_id.parse()?;

            let (character, light_cone) =
                if pull.item_type == "Character" || pull.item_type == "角色" {
                    (Some(item_id), None)
                } else if pull.item_type == "Light Cone"
                    || pull.item_type == "光锥"
                    || pull.item_type == "光錐"
                {
                    (None, Some(item_id))
                } else {
                    return Ok(HttpResponse::BadRequest().finish());
                };

            warps_map
                .entry(gacha_type)
                .or_default()
                .push((id, character, light_cone, time));
        }

        let mut set_all_departure = database::warps::SetAll::default();
        let mut set_all_standard = database::warps::SetAll::default();
        let mut set_all_special = database::warps::SetAll::default();
        let mut set_all_lc = database::warps::SetAll::default();
        let mut set_all_collab = database::warps::SetAll::default();
        let mut set_all_collab_lc = database::warps::SetAll::default();

        for (gacha_type, pulls) in &warps_map {
            let earliest_timestamp = match gacha_type {
                GachaType::Departure => {
                    database::warps::departure::get_earliest_timestamp_by_uid(uid, &pool).await?
                }
                GachaType::Standard => {
                    database::warps::standard::get_earliest_timestamp_by_uid(uid, &pool).await?
                }
                GachaType::Special => {
                    database::warps::special::get_earliest_timestamp_by_uid(uid, &pool).await?
                }
                GachaType::Lc => {
                    database::warps::lc::get_earliest_timestamp_by_uid(uid, &pool).await?
                }
                GachaType::Collab => {
                    database::warps::collab::get_earliest_timestamp_by_uid(uid, &pool).await?
                }
                GachaType::CollabLc => {
                    database::warps::collab_lc::get_earliest_timestamp_by_uid(uid, &pool).await?
                }
            };

            for (id, character, light_cone, time) in pulls {
                if !admin {
                    if let Some(earliest_timestamp) = earliest_timestamp {
                        if *time >= earliest_timestamp {
                            break;
                        }
                    }
                }
                let set_all = match gacha_type {
                    GachaType::Departure => &mut set_all_departure,
                    GachaType::Standard => &mut set_all_standard,
                    GachaType::Special => &mut set_all_special,
                    GachaType::Lc => &mut set_all_lc,
                    GachaType::Collab => &mut set_all_collab,
                    GachaType::CollabLc => &mut set_all_collab_lc,
                };

                set_all.id.push(*id);
                set_all.uid.push(uid);
                set_all.character.push(*character);
                set_all.light_cone.push(*light_cone);
                set_all.timestamp.push(*time);
                set_all.official.push(false);
            }
        }

        for (pull_pool, set) in [
            (GachaType::Departure, &set_all_departure),
            (GachaType::Standard, &set_all_standard),
            (GachaType::Special, &set_all_special),
            (GachaType::Lc, &set_all_lc),
            (GachaType::Collab, &set_all_collab),
            (GachaType::CollabLc, &set_all_collab_lc),
        ] {
            normalized_pulls.extend(crate::gacha::imports::normalize_hsr_set(pull_pool, set)?);
        }
    }

    // ZZZ (nap)
    for entry in &uigf.nap {
        let uid = entry.uid.clone().parse()?;
        if !check_zzz_auth(admin, &username, uid, &pool).await? {
            zzz_skipped.skipped_uids += 1;
            zzz_skipped.skipped_records += entry.list.len() as u64;
            continue;
        }

        let tz = FixedOffset::east_opt(3600 * entry.timezone).unwrap();

        let mut signals_map: HashMap<
            i32,
            Vec<(i64, Option<i32>, Option<i32>, Option<i32>, DateTime<Utc>)>,
        > = HashMap::new();

        for pull in &entry.list {
            let Some(gacha_type) = ZzzGachaType::from_uigf_id(&pull.gacha_type) else {
                return Ok(HttpResponse::BadRequest().finish());
            };
            let gacha_type_id = gacha_type.id();

            let time = NaiveDateTime::parse_from_str(&pull.time, "%Y-%m-%d %H:%M:%S")?
                .and_local_timezone(tz)
                .unwrap()
                .to_utc();

            let id: i64 = pull.id.parse()?;
            let item_id: i32 = pull.item_id.parse()?;

            let (character, w_engine, bangboo) = if pull.item_type == "Character"
                || pull.item_type == "Agents"
                || pull.item_type == "代理人"
            {
                (Some(item_id), None, None)
            } else if pull.item_type == "W-Engines" || pull.item_type == "音擎" {
                (None, Some(item_id), None)
            } else if pull.item_type == "Bangboo" || pull.item_type == "邦布" {
                (None, None, Some(item_id))
            } else {
                return Ok(HttpResponse::BadRequest().finish());
            };

            signals_map
                .entry(gacha_type_id)
                .or_default()
                .push((id, character, w_engine, bangboo, time));
        }

        let mut set_all_standard = database::zzz::signals::SetAll::default();
        let mut set_all_special = database::zzz::signals::SetAll::default();
        let mut set_all_w_engine = database::zzz::signals::SetAll::default();
        let mut set_all_bangboo = database::zzz::signals::SetAll::default();
        let mut set_all_exclusive_rescreening = database::zzz::signals::SetAll::default();
        let mut set_all_w_engine_reverberation = database::zzz::signals::SetAll::default();

        for (gacha_type_id, pulls) in &signals_map {
            let gacha_type = match *gacha_type_id {
                x if x == ZzzGachaType::Standard.id() => ZzzGachaType::Standard,
                x if x == ZzzGachaType::Special.id() => ZzzGachaType::Special,
                x if x == ZzzGachaType::WEngine.id() => ZzzGachaType::WEngine,
                x if x == ZzzGachaType::Bangboo.id() => ZzzGachaType::Bangboo,
                x if x == ZzzGachaType::ExclusiveRescreening.id() => {
                    ZzzGachaType::ExclusiveRescreening
                }
                x if x == ZzzGachaType::WEngineReverberation.id() => {
                    ZzzGachaType::WEngineReverberation
                }
                _ => return Ok(HttpResponse::BadRequest().finish()),
            };
            let earliest_timestamp = match gacha_type {
                ZzzGachaType::Standard => {
                    database::zzz::signals::standard::get_earliest_timestamp_by_uid(uid, &pool)
                        .await?
                }
                ZzzGachaType::Special => {
                    database::zzz::signals::special::get_earliest_timestamp_by_uid(uid, &pool)
                        .await?
                }
                ZzzGachaType::WEngine => {
                    database::zzz::signals::w_engine::get_earliest_timestamp_by_uid(uid, &pool)
                        .await?
                }
                ZzzGachaType::Bangboo => {
                    database::zzz::signals::bangboo::get_earliest_timestamp_by_uid(uid, &pool)
                        .await?
                }
                ZzzGachaType::ExclusiveRescreening => {
                    database::zzz::signals::exclusive_rescreening::get_earliest_timestamp_by_uid(
                        uid, &pool,
                    )
                    .await?
                }
                ZzzGachaType::WEngineReverberation => {
                    database::zzz::signals::w_engine_reverberation::get_earliest_timestamp_by_uid(
                        uid, &pool,
                    )
                    .await?
                }
            };
            for (id, character, w_engine, bangboo, time) in pulls {
                if !admin && earliest_timestamp.is_some_and(|earliest| *time >= earliest) {
                    break;
                }
                let set_all = match *gacha_type_id {
                    x if x == ZzzGachaType::Standard.id() => &mut set_all_standard,
                    x if x == ZzzGachaType::Special.id() => &mut set_all_special,
                    x if x == ZzzGachaType::WEngine.id() => &mut set_all_w_engine,
                    x if x == ZzzGachaType::Bangboo.id() => &mut set_all_bangboo,
                    x if x == ZzzGachaType::ExclusiveRescreening.id() => {
                        &mut set_all_exclusive_rescreening
                    }
                    x if x == ZzzGachaType::WEngineReverberation.id() => {
                        &mut set_all_w_engine_reverberation
                    }
                    _ => return Ok(HttpResponse::BadRequest().finish()),
                };

                set_all.id.push(*id);
                set_all.uid.push(uid);
                set_all.character.push(*character);
                set_all.w_engine.push(*w_engine);
                set_all.bangboo.push(*bangboo);
                set_all.timestamp.push(*time);
                set_all.official.push(false);
            }
        }

        for (pull_pool, set) in [
            (ZzzGachaType::Standard, &set_all_standard),
            (ZzzGachaType::Special, &set_all_special),
            (ZzzGachaType::WEngine, &set_all_w_engine),
            (ZzzGachaType::Bangboo, &set_all_bangboo),
            (
                ZzzGachaType::ExclusiveRescreening,
                &set_all_exclusive_rescreening,
            ),
            (
                ZzzGachaType::WEngineReverberation,
                &set_all_w_engine_reverberation,
            ),
        ] {
            normalized_pulls.extend(crate::gacha::imports::normalize_zzz_set(pull_pool, set)?);
        }
    }

    // GI (hk4e)
    for entry in &uigf.hk4e {
        let uid = entry.uid.clone().parse()?;
        if !check_gi_auth(admin, &username, uid, &pool).await? {
            gi_skipped.skipped_uids += 1;
            gi_skipped.skipped_records += entry.list.len() as u64;
            continue;
        }

        let tz = FixedOffset::east_opt(3600 * entry.timezone).unwrap();

        let mut wishes_map: HashMap<
            GiGachaType,
            Vec<(i64, Option<i32>, Option<i32>, DateTime<Utc>)>,
        > = HashMap::new();

        for pull in &entry.list {
            let gacha_type = match pull.uigf_gacha_type.as_str() {
                "100" => GiGachaType::Beginner,
                "200" => GiGachaType::Standard,
                "301" => GiGachaType::Character,
                "302" => GiGachaType::Weapon,
                "500" => GiGachaType::Chronicled,
                _ => return Ok(HttpResponse::BadRequest().finish()),
            };

            let time = NaiveDateTime::parse_from_str(&pull.time, "%Y-%m-%d %H:%M:%S")?
                .and_local_timezone(tz)
                .unwrap()
                .to_utc();

            let id: i64 = pull.id.parse()?;
            let item_id: i32 = pull.item_id.parse()?;

            let (character, weapon) = if pull.item_type == "Character" || pull.item_type == "角色"
            {
                (Some(item_id), None)
            } else if pull.item_type == "Weapon"
                || pull.item_type == "Weapons"
                || pull.item_type == "武器"
            {
                (None, Some(item_id))
            } else {
                return Ok(HttpResponse::BadRequest().finish());
            };

            wishes_map
                .entry(gacha_type)
                .or_default()
                .push((id, character, weapon, time));
        }

        let mut set_all_beginner = database::gi::wishes::SetAll::default();
        let mut set_all_standard = database::gi::wishes::SetAll::default();
        let mut set_all_character = database::gi::wishes::SetAll::default();
        let mut set_all_weapon = database::gi::wishes::SetAll::default();
        let mut set_all_chronicled = database::gi::wishes::SetAll::default();

        for (gacha_type, pulls) in &wishes_map {
            let earliest_timestamp = match gacha_type {
                GiGachaType::Beginner => {
                    database::gi::wishes::beginner::get_earliest_timestamp_by_uid(uid, &pool)
                        .await?
                }
                GiGachaType::Standard => {
                    database::gi::wishes::standard::get_earliest_timestamp_by_uid(uid, &pool)
                        .await?
                }
                GiGachaType::Character => {
                    database::gi::wishes::character::get_earliest_timestamp_by_uid(uid, &pool)
                        .await?
                }
                GiGachaType::Weapon => {
                    database::gi::wishes::weapon::get_earliest_timestamp_by_uid(uid, &pool).await?
                }
                GiGachaType::Chronicled => {
                    database::gi::wishes::chronicled::get_earliest_timestamp_by_uid(uid, &pool)
                        .await?
                }
            };

            for (id, character, weapon, time) in pulls {
                if !admin {
                    if let Some(earliest_timestamp) = earliest_timestamp {
                        if *time >= earliest_timestamp {
                            break;
                        }
                    }
                }
                let set_all = match gacha_type {
                    GiGachaType::Beginner => &mut set_all_beginner,
                    GiGachaType::Standard => &mut set_all_standard,
                    GiGachaType::Character => &mut set_all_character,
                    GiGachaType::Weapon => &mut set_all_weapon,
                    GiGachaType::Chronicled => &mut set_all_chronicled,
                };

                set_all.id.push(*id);
                set_all.uid.push(uid);
                set_all.character.push(*character);
                set_all.weapon.push(*weapon);
                set_all.timestamp.push(*time);
                set_all.official.push(false);
            }
        }

        for (pull_pool, set) in [
            (GiGachaType::Beginner, &set_all_beginner),
            (GiGachaType::Standard, &set_all_standard),
            (GiGachaType::Character, &set_all_character),
            (GiGachaType::Weapon, &set_all_weapon),
            (GiGachaType::Chronicled, &set_all_chronicled),
        ] {
            normalized_pulls.extend(crate::gacha::imports::normalize_gi_set(pull_pool, set)?);
        }
    }

    let batch = crate::gacha::imports::ImportBatch::new(
        normalized_pulls,
        crate::gacha::imports::ImportPolicy::unofficial(admin, true, admin),
    )?;
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
