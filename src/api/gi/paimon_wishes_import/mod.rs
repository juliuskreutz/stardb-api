use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use reqwest::header;
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, GiGachaType};

#[derive(OpenApi)]
#[openapi(
    tags((name = "paimon-wishes-import")),
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
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Paimon {
    wish_uid: String,
    wish_counter_beginners: Wishes,
    wish_counter_standard: Wishes,
    wish_counter_character_event: Wishes,
    wish_counter_weapon_event: Wishes,
    wish_counter_chronicled: Wishes,
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
}

#[utoipa::path(
    tag = "paimon-wishes-import",
    post,
    path = "/api/paimon-wishes-import",
    request_body = PaimonWishesImportParams,
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[post("/api/paimon-wishes-import")]
async fn post_paimon_warps_import(
    session: Session,
    params: web::Json<PaimonWishesImportParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::admins::get_one_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let paimon: Paimon = serde_json::from_str(&params.data)?;

    let uid = paimon.wish_uid.parse()?;

    let name = reqwest::Client::new()
        .get(format!("https://enka.network/api/uid/{uid}?info"))
        .header(header::USER_AGENT, "stardb")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?["playerInfo"]["nickname"]
        .as_str()
        .unwrap()
        .to_string();

    database::gi::profiles::set(&database::gi::profiles::DbProfile { uid, name }, &pool).await?;
    if let Ok(Some(username)) = session.get::<String>("username") {
        let connection = database::gi::connections::DbConnection {
            uid,
            username,
            verified: true,
            private: false,
        };

        database::gi::connections::set(&connection, &pool).await?;
    }

    let timestamp_offset = chrono::Duration::hours(match uid.to_string().chars().next() {
        Some('6') => -5,
        Some('7') => 1,
        _ => 8,
    });

    let paimon: Paimon = serde_json::from_str(&params.data)?;

    let mut set_all_beginner = database::gi::wishes::SetAll::default();
    let mut set_all_standard = database::gi::wishes::SetAll::default();
    let mut set_all_character = database::gi::wishes::SetAll::default();
    let mut set_all_weapon = database::gi::wishes::SetAll::default();
    let mut set_all_chronicled = database::gi::wishes::SetAll::default();

    for (wishes, gacha_type) in [
        (&paimon.wish_counter_beginners, GiGachaType::Beginner),
        (&paimon.wish_counter_standard, GiGachaType::Standard),
        (&paimon.wish_counter_character_event, GiGachaType::Character),
        (&paimon.wish_counter_weapon_event, GiGachaType::Weapon),
        (&paimon.wish_counter_chronicled, GiGachaType::Chronicled),
    ] {
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

        for (id, wish) in wishes.pulls.iter().enumerate() {
            info!("{}", wish.id);
            let (character, weapon) = match wish.r#type.as_str() {
                "character" => (
                    Some(database::gi::characters::get_id_by_paimon_moe_id(&wish.id, &pool).await?),
                    None,
                ),
                "weapon" => (
                    None,
                    Some(database::gi::weapons::get_id_by_paimon_moe_id(&wish.id, &pool).await?),
                ),
                _ => return Ok(HttpResponse::BadRequest().finish()),
            };

            let timestamp = NaiveDateTime::parse_from_str(&wish.time, "%Y-%m-%d %H:%M:%S")?
                .and_utc()
                - timestamp_offset;

            if let Some(earliest_timestamp) = earliest_timestamp {
                if timestamp >= earliest_timestamp {
                    break;
                }
            }

            let set_all = match gacha_type {
                GiGachaType::Beginner => &mut set_all_beginner,
                GiGachaType::Standard => &mut set_all_standard,
                GiGachaType::Character => &mut set_all_character,
                GiGachaType::Weapon => &mut set_all_weapon,
                GiGachaType::Chronicled => &mut set_all_chronicled,
            };

            set_all.id.push(id as i64);
            set_all.uid.push(uid);
            set_all.character.push(character);
            set_all.weapon.push(weapon);
            set_all.timestamp.push(timestamp);
            set_all.official.push(false);
        }
    }

    database::gi::wishes::beginner::set_all(&set_all_beginner, &pool).await?;
    database::gi::wishes::standard::set_all(&set_all_standard, &pool).await?;
    database::gi::wishes::character::set_all(&set_all_character, &pool).await?;
    database::gi::wishes::weapon::set_all(&set_all_weapon, &pool).await?;
    database::gi::wishes::chronicled::set_all(&set_all_chronicled, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
