use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use rand::seq::IndexedRandom as _;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, GiGachaType};

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/paimon-wishes-import")),
    paths(post_paimon_warps_import),
    components(schemas(PaimonWishesImportParams)),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

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
        uid as i32
    } else {
        wish_uid.as_str().unwrap().parse()?
    };

    let admin = database::admins::exists(&username, &pool).await?;

    if !admin
        && database::gi::profiles::get_by_uid(uid, &pool)
            .await
            .is_err()
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

    let db_weapons = database::gi::weapons::get_all(&pool).await?;

    let weapons_3_ids: Vec<_> = db_weapons
        .iter()
        .filter_map(|w| (w.rarity == 3).then_some(w.id))
        .collect();

    let weapons_4_ids: Vec<_> = db_weapons
        .iter()
        .filter_map(|w| (w.rarity == 4).then_some(w.id))
        .collect();

    let mut set_all_beginner = database::gi::wishes::SetAll::default();
    let mut set_all_standard = database::gi::wishes::SetAll::default();
    let mut set_all_character = database::gi::wishes::SetAll::default();
    let mut set_all_weapon = database::gi::wishes::SetAll::default();
    let mut set_all_chronicled = database::gi::wishes::SetAll::default();

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

        let earliest_timestamp = match gacha_type {
            GiGachaType::Beginner => {
                database::gi::wishes::beginner::get_earliest_timestamp_by_uid(uid, &pool).await?
            }
            GiGachaType::Standard => {
                database::gi::wishes::standard::get_earliest_timestamp_by_uid(uid, &pool).await?
            }
            GiGachaType::Character => {
                database::gi::wishes::character::get_earliest_timestamp_by_uid(uid, &pool).await?
            }
            GiGachaType::Weapon => {
                database::gi::wishes::weapon::get_earliest_timestamp_by_uid(uid, &pool).await?
            }
            GiGachaType::Chronicled => {
                database::gi::wishes::chronicled::get_earliest_timestamp_by_uid(uid, &pool).await?
            }
        };

        let mut pity_4 = 1;
        let mut pity_5 = 1;

        let mut id = 0;
        for wish in wishes.pulls.iter() {
            let timestamp =
                NaiveDateTime::parse_from_str(&wish.time, "%Y-%m-%d %H:%M:%S")?.and_utc();

            if let Some(earliest_timestamp) = earliest_timestamp {
                if timestamp >= earliest_timestamp {
                    break;
                }
            }

            let (character, weapon, rarity) = match wish.id.as_str() {
                "unknown_3_star" => (None, weapons_3_ids.choose(&mut rand::rng()).copied(), 3),
                "unknown_4_star" => (None, weapons_4_ids.choose(&mut rand::rng()).copied(), 4),
                _ => match wish.r#type.as_str() {
                    "character" => {
                        let character =
                            database::gi::characters::get_by_paimon_moe_id(&wish.id, &pool).await?;
                        (Some(character.id), None, character.rarity)
                    }
                    "weapon" => {
                        let weapon =
                            database::gi::weapons::get_by_paimon_moe_id(&wish.id, &pool).await?;
                        (None, Some(weapon.id), weapon.rarity)
                    }
                    _ => return Ok(HttpResponse::BadRequest().finish()),
                },
            };

            let set_all = match gacha_type {
                GiGachaType::Beginner => &mut set_all_beginner,
                GiGachaType::Standard => &mut set_all_standard,
                GiGachaType::Character => &mut set_all_character,
                GiGachaType::Weapon => &mut set_all_weapon,
                GiGachaType::Chronicled => &mut set_all_chronicled,
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
                    set_all.id.push(id);
                    set_all.uid.push(uid);
                    set_all.character.push(None);
                    set_all.timestamp.push(timestamp);
                    set_all.official.push(false);

                    if pity_4 < 10 {
                        let id = weapons_3_ids.choose(&mut rand::rng()).copied();

                        set_all.weapon.push(id);

                        pity_4 += 1;
                    } else {
                        let id = weapons_4_ids.choose(&mut rand::rng()).copied();

                        set_all.weapon.push(id);

                        pity_4 = 1;
                    }

                    id += 1;
                    pity += 1;
                }

                pity_4 = 1;
            }

            set_all.id.push(id);
            set_all.uid.push(uid);
            set_all.character.push(character);
            set_all.weapon.push(weapon);
            set_all.timestamp.push(timestamp);
            set_all.official.push(false);

            id += 1;
        }
    }

    database::gi::wishes::beginner::set_all(&set_all_beginner, &pool).await?;
    database::gi::wishes::standard::set_all(&set_all_standard, &pool).await?;
    database::gi::wishes::character::set_all(&set_all_character, &pool).await?;
    database::gi::wishes::weapon::set_all(&set_all_weapon, &pool).await?;
    database::gi::wishes::chronicled::set_all(&set_all_chronicled, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
