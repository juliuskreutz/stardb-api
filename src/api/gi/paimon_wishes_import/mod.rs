use crate::gacha::imports::{ImportBatch, NormalizedPull, PullItem, PullPool, PullProvenance};
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use rand::seq::IndexedRandom as _;
use sqlx::PgPool;
use std::collections::HashMap;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, GiGachaType};

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/paimon-wishes-import")),
    paths(post_paimon_warps_import),
    components(schemas(PaimonWishesImportParams)),
)]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module's routes and any shared application data.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_paimon_warps_import);
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
struct PaimonWishesImportParams {
    data: String,
    profile: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Wishes {
    pulls: Vec<Pull>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Pull {
    r#type: String,
    id: String,
    time: String,
    pity: usize,
}

#[utoipa::path(
    tag = "gi/paimon-wishes-import",
    post,
    path = "/api/gi/paimon-wishes-import",
    request_body = PaimonWishesImportParams,
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[post("/api/gi/paimon-wishes-import")]
/// Imports a claimed Paimon profile as unofficial GI pulls and recalculates stats.
/// Every pool stops at its first overlap, including for admins. Missing pity entries
/// are filled from the weapon catalog; other parsing or storage failures propagate.
async fn post_paimon_warps_import(
    session: Session,
    params: web::Json<PaimonWishesImportParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let json: serde_json::Value = serde_json::from_str(&params.data)?;

    let wish_uid = json[format!("{}wish-uid", params.profile)].clone();

    let uid = if let Some(uid) = wish_uid.as_i64() {
        let Ok(uid) = i32::try_from(uid) else {
            return Ok(HttpResponse::BadRequest().finish());
        };
        uid
    } else {
        let Some(uid) = wish_uid
            .as_str()
            .and_then(|value| value.parse::<i32>().ok())
        else {
            return Ok(HttpResponse::BadRequest().finish());
        };
        uid
    };

    let admin = database::admins::exists(&username, &pool).await?;

    if !admin
        && database::gi::profiles::get_by_uid(uid, &pool)
            .await?
            .is_none()
    {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let allowed = admin
        || database::gi::connections::get_by_username(&username, &pool)
            .await?
            .iter()
            .find(|c| c.uid == uid)
            .map(|c| c.verified)
            .unwrap_or_default();

    if !allowed {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let wish_counter_beginners: Option<Wishes> =
        serde_json::from_value(json[format!("{}wish-counter-beginners", params.profile)].clone())?;
    let wish_counter_standard: Option<Wishes> =
        serde_json::from_value(json[format!("{}wish-counter-standard", params.profile)].clone())?;
    let wish_counter_character_event: Option<Wishes> = serde_json::from_value(
        json[format!("{}wish-counter-character-event", params.profile)].clone(),
    )?;
    let wish_counter_weapon_event: Option<Wishes> = serde_json::from_value(
        json[format!("{}wish-counter-weapon-event", params.profile)].clone(),
    )?;
    let wish_counter_chronicled: Option<Wishes> =
        serde_json::from_value(json[format!("{}wish-counter-chronicled", params.profile)].clone())?;

    let timestamp_offset = chrono::Duration::hours(match uid.to_string().chars().next() {
        Some('6') => -5,
        Some('7') => 1,
        _ => 8,
    });

    let db_weapons = database::gi::weapons::get_all(&pool).await?;

    let weapons_3_ids: Vec<_> = db_weapons
        .iter()
        .filter_map(|w| (w.rarity == 3).then_some(w.id))
        .collect();

    let weapons_4_ids: Vec<_> = db_weapons
        .iter()
        .filter_map(|w| (w.rarity == 4).then_some(w.id))
        .collect();

    let mut pulls = Vec::new();
    // Share successful named resolutions across pools, only for this import request.
    let mut resolved_items = HashMap::new();

    for (wishes, gacha_type) in [
        (&wish_counter_beginners, GiGachaType::Beginner),
        (&wish_counter_standard, GiGachaType::Standard),
        (&wish_counter_character_event, GiGachaType::Character),
        (&wish_counter_weapon_event, GiGachaType::Weapon),
        (&wish_counter_chronicled, GiGachaType::Chronicled),
    ] {
        let Some(wishes) = wishes else {
            continue;
        };

        let earliest_timestamp =
            database::gi::wishes::get_earliest_timestamp_by_uid_by_pool(gacha_type, uid, &pool)
                .await?;

        let mut pity_4 = 1;
        let mut pity_5 = 1;

        let mut id = 0;
        for wish in wishes.pulls.iter() {
            let timestamp = NaiveDateTime::parse_from_str(&wish.time, "%Y-%m-%d %H:%M:%S")?
                .and_utc()
                .checked_sub_signed(timestamp_offset)
                .ok_or_else(|| anyhow::anyhow!("invalid timestamp offset"))?;

            if let Some(earliest_timestamp) = earliest_timestamp {
                if timestamp >= earliest_timestamp {
                    break;
                }
            }

            let Some((item, rarity)) = resolve_item(
                &wish.r#type,
                &wish.id,
                &weapons_3_ids,
                &weapons_4_ids,
                &mut resolved_items,
                &pool,
            )
            .await?
            else {
                return Ok(HttpResponse::BadRequest().finish());
            };

            let mut pity = 1;
            match rarity {
                3 => {
                    pity_4 += 1;
                    pity_5 += 1;
                }
                4 => {
                    pity = pity_4;
                    pity_4 = 1;
                    pity_5 += 1;
                }
                5 => {
                    pity = pity_5;
                    pity_4 += 1;
                    pity_5 = 1;
                }
                _ => return Ok(HttpResponse::BadRequest().finish()),
            }

            if rarity == 5 {
                while pity < wish.pity {
                    let filler = if pity_4 < 10 {
                        pity_4 += 1;
                        random_weapon(&weapons_3_ids)?
                    } else {
                        pity_4 = 1;
                        random_weapon(&weapons_4_ids)?
                    };
                    pulls.push(NormalizedPull {
                        uid,
                        id,
                        pool: PullPool::Gi(gacha_type),
                        item: filler,
                        timestamp,
                        provenance: PullProvenance::Unofficial,
                    });

                    id += 1;
                    pity += 1;
                }

                pity_4 = 1;
            }

            pulls.push(NormalizedPull {
                uid,
                id,
                pool: PullPool::Gi(gacha_type),
                item,
                timestamp,
                provenance: PullProvenance::Unofficial,
            });

            id += 1;
        }
    }

    let Ok(batch) = ImportBatch::new(pulls, PullProvenance::Unofficial) else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    crate::gacha::imports::persist_batch_in_transaction(&batch, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

/// Samples a weapon ID for reconstructed history, failing if its rarity catalog is empty.
fn random_weapon(ids: &[i32]) -> anyhow::Result<PullItem> {
    ids.choose(&mut rand::rng())
        .copied()
        .map(PullItem::Weapon)
        .ok_or_else(|| anyhow::anyhow!("missing weapon catalog"))
}

/// Resolve each named item once per request. Unknown entries must sample on every
/// occurrence, and failures are never cached. Item type separates identical names.
async fn resolve_item(
    item_type: &str,
    id: &str,
    weapons_3: &[i32],
    weapons_4: &[i32],
    resolved: &mut HashMap<(String, String), (PullItem, i32)>,
    pool: &PgPool,
) -> anyhow::Result<Option<(PullItem, i32)>> {
    match id {
        "unknown_3_star" => return Ok(Some((random_weapon(weapons_3)?, 3))),
        "unknown_4_star" => return Ok(Some((random_weapon(weapons_4)?, 4))),
        _ => {}
    }
    let key = (item_type.to_owned(), id.to_owned());
    if let Some(item) = resolved.get(&key) {
        return Ok(Some(*item));
    }
    let item = match item_type {
        "character" => {
            let item = database::gi::characters::get_by_paimon_moe_id(id, pool).await?;
            (PullItem::Character(item.id), item.rarity)
        }
        "weapon" => {
            let item = database::gi::weapons::get_by_paimon_moe_id(id, pool).await?;
            (PullItem::Weapon(item.id), item.rarity)
        }
        _ => return Ok(None),
    };
    resolved.insert(key, item);
    Ok(Some(item))
}

#[cfg(test)]
mod resolution_tests {
    use super::*;

    #[sqlx::test]
    async fn sql_performance_paimon_reuses_known_items_only(pool: PgPool) {
        sqlx::query("INSERT INTO gi_characters(id, rarity) VALUES (1, 5)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO gi_weapons(id, rarity) VALUES (2, 4)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO gi_characters_text(id, language, name) VALUES (1, 'en', 'Seed Name')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO gi_weapons_text(id, language, name) VALUES (2, 'en', 'Seed Name')",
        )
        .execute(&pool)
        .await
        .unwrap();
        let mut cache = HashMap::new();
        assert_eq!(
            resolve_item("character", "seed_name", &[], &[], &mut cache, &pool)
                .await
                .unwrap(),
            Some((PullItem::Character(1), 5))
        );
        assert_eq!(
            resolve_item("weapon", "seed_name", &[], &[], &mut cache, &pool)
                .await
                .unwrap(),
            Some((PullItem::Weapon(2), 4))
        );
        assert_eq!(cache.len(), 2);
        assert!(
            resolve_item("weapon", "missing", &[], &[], &mut cache, &pool)
                .await
                .is_err()
        );
        assert_eq!(cache.len(), 2);
        // A closed pool proves repeat resolutions need no database connection.
        pool.close().await;
        for _ in 0..1000 {
            assert_eq!(
                resolve_item("character", "seed_name", &[], &[], &mut cache, &pool)
                    .await
                    .unwrap(),
                Some((PullItem::Character(1), 5))
            );
        }
        // Different singleton catalogs make accidental unknown-item caching deterministic.
        for id in [10, 11] {
            assert_eq!(
                resolve_item("weapon", "unknown_3_star", &[id], &[], &mut cache, &pool)
                    .await
                    .unwrap(),
                Some((PullItem::Weapon(id), 3))
            );
            assert_eq!(
                resolve_item("weapon", "unknown_4_star", &[], &[id], &mut cache, &pool)
                    .await
                    .unwrap(),
                Some((PullItem::Weapon(id), 4))
            );
        }
        assert_eq!(cache.len(), 2);
        assert!(
            resolve_item("weapon", "unknown_3_star", &[], &[], &mut cache, &pool)
                .await
                .is_err()
        );
        assert!(
            resolve_item("invalid", "seed_name", &[], &[], &mut cache, &pool)
                .await
                .unwrap()
                .is_none()
        );
    }
}
