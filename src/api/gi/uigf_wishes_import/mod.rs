use std::collections::HashMap;

use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use sqlx::PgPool;
use strum::IntoEnumIterator;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, GiGachaType};

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/uigf-wishes-import")),
    paths(post_uigf_warps_import),
    components(schemas(UigfWishesImportParams)),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_uigf_warps_import);
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
struct UigfWishesImportParams {
    data: String,
}

#[derive(serde::Deserialize)]
struct Uigf {
    hk4e: Vec<Hk4e>,
}

#[derive(serde::Deserialize)]
struct Hk4e {
    uid: String,
    timezone: i32,
    list: Vec<Pull>,
}

#[derive(serde::Deserialize)]
struct Pull {
    id: String,
    uigf_gacha_type: String,
    item_id: String,
    item_type: String,
    time: String,
}

struct ParsedWarp {
    id: i64,
    item_id: i32,
    item_type: String,
    time: DateTime<Utc>,
}

#[utoipa::path(
    tag = "gi/uigf-wishes-import",
    post,
    path = "/api/gi/uigf-wishes-import",
    request_body = UigfWishesImportParams,
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[post("/api/gi/uigf-wishes-import")]
async fn post_uigf_warps_import(
    session: Session,
    params: web::Json<UigfWishesImportParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let uigf: Uigf = serde_json::from_str(&params.data)?;

    for hk4e in uigf.hk4e {
        let uid = hk4e.uid.parse()?;

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

        let mut wishes_map: HashMap<_, Vec<ParsedWarp>> = HashMap::new();
        let tz = FixedOffset::east_opt(3600 * hk4e.timezone).unwrap();

        for pull in hk4e.list {
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

            wishes_map.entry(gacha_type).or_default().push(ParsedWarp {
                id: pull.id.parse()?,
                item_id: pull.item_id.parse()?,
                item_type: pull.item_type,
                time,
            });
        }

        let mut set_all_beginner = database::gi::wishes::SetAll::default();
        let mut set_all_standard = database::gi::wishes::SetAll::default();
        let mut set_all_character = database::gi::wishes::SetAll::default();
        let mut set_all_weapon = database::gi::wishes::SetAll::default();
        let mut set_all_chronicled = database::gi::wishes::SetAll::default();

        for gacha_type in GiGachaType::iter() {
            let Some(wishes) = wishes_map.get(&gacha_type) else {
                continue;
            };

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

            for wish in wishes {
                let timestamp = wish.time;

                if !admin {
                    if let Some(earliest_timestamp) = earliest_timestamp {
                        if timestamp >= earliest_timestamp {
                            break;
                        }
                    }
                }

                let id = wish.id;
                let item_id = wish.item_id;

                let (character, light_cone) =
                    if wish.item_type == "Character" || wish.item_type == "角色" {
                        (Some(item_id), None)
                    } else if wish.item_type == "Weapon"
                        || wish.item_type == "Weapons"  // Weapons should never happen but it does not hurt
                        || wish.item_type == "武器"
                    {
                        (None, Some(item_id))
                    } else {
                        return Ok(HttpResponse::BadRequest().finish());
                    };

                let set_all = match gacha_type {
                    GiGachaType::Beginner => &mut set_all_beginner,
                    GiGachaType::Standard => &mut set_all_standard,
                    GiGachaType::Character => &mut set_all_character,
                    GiGachaType::Weapon => &mut set_all_weapon,
                    GiGachaType::Chronicled => &mut set_all_chronicled,
                };

                set_all.id.push(id);
                set_all.uid.push(uid);
                set_all.character.push(character);
                set_all.weapon.push(light_cone);
                set_all.timestamp.push(timestamp);
                set_all.official.push(false);
            }
        }

        database::gi::wishes::beginner::set_all(&set_all_beginner, &pool).await?;
        database::gi::wishes::standard::set_all(&set_all_standard, &pool).await?;
        database::gi::wishes::character::set_all(&set_all_character, &pool).await?;
        database::gi::wishes::weapon::set_all(&set_all_weapon, &pool).await?;
        database::gi::wishes::chronicled::set_all(&set_all_chronicled, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
