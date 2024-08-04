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

    let mut set_all_departure = database::warps::SetAll::default();
    let mut set_all_standard = database::warps::SetAll::default();
    let mut set_all_special = database::warps::SetAll::default();
    let mut set_all_lc = database::warps::SetAll::default();

    for (wishes, gacha_type) in [
        (&paimon.wish_counter_beginners, GiGachaType::Beginner),
        (&paimon.wish_counter_standard, GiGachaType::Standard),
        (&paimon.wish_counter_character_event, GiGachaType::Character),
        (&paimon.wish_counter_weapon_event, GiGachaType::Weapon),
        (&paimon.wish_counter_chronicled, GiGachaType::Chronicled),
    ] {
        for wish in wishes.pulls {
            let id = wish.id.parse::<i64>().unwrap();
            let item_id = wish.item_id.parse().unwrap();
            let (character, light_cone) = if item_id < 2000 {
                (Some(item_id), None)
            } else {
                (None, Some(item_id))
            };

            let timestamp = NaiveDateTime::parse_from_str(&wish.time, "%Y-%m-%d %H:%M:%S")?
                .and_utc()
                - timestamp_offset;

            let set_all = match gacha_type {
                GachaType::Departure => &mut set_all_departure,
                GachaType::Standard => &mut set_all_standard,
                GachaType::Special => &mut set_all_special,
                GachaType::Lc => &mut set_all_lc,
            };

            set_all.id.push(id);
            set_all.uid.push(uid);
            set_all.character.push(character);
            set_all.light_cone.push(light_cone);
            set_all.timestamp.push(timestamp);
            set_all.official.push(false);
        }
    }

    database::warps::departure::set_all(&set_all_departure, &pool).await?;
    database::warps::standard::set_all(&set_all_standard, &pool).await?;
    database::warps::special::set_all(&set_all_special, &pool).await?;
    database::warps::lc::set_all(&set_all_lc, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
