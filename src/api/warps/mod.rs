mod uid;

use actix_web::{post, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps")),
    paths(post_warps),
    components(schemas(WarpAuthKey))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(uid::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_warps).configure(uid::configure);
}

#[derive(Deserialize)]
struct GachaLog {
    data: Data,
}

#[derive(Deserialize)]
struct Data {
    list: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    uid: String,
    item_type: String,
    item_id: String,
    time: String,
}

#[derive(Deserialize, ToSchema)]
struct WarpAuthKey {
    auth_key: String,
}

#[utoipa::path(
    tag = "warps",
    post,
    path = "/api/warps",
    request_body = WarpAuthKey,
    responses(
        (status = 200, description = "Success"),
    )
)]
#[post("/api/warps")]
async fn post_warps(
    auth_key: web::Json<WarpAuthKey>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let auth_key = urlencoding::encode(&auth_key.auth_key).to_string();
    let url = format!("https://api-os-takumi.mihoyo.com/common/gacha_record/api/getGachaLog?authkey_ver=1&sign_type=2&lang=en&authkey={auth_key}&game_biz=hkrpg_global&size=20");

    let mut end_id = None;

    //Standard Warp
    'outer: loop {
        let gacha_log: GachaLog = reqwest::get(format!(
            "{url}{}&gacha_type=1",
            end_id.map(|id| format!("&end_id={id}")).unwrap_or_default()
        ))
        .await?
        .json()
        .await?;

        if gacha_log.data.list.is_empty() {
            break;
        }

        for entry in gacha_log.data.list.iter() {
            let id = entry.id.parse()?;
            let uid = entry.uid.parse()?;

            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?;

            if entry.item_type == "Character" {
                if database::get_warp_standard_character_by_id_and_uid(id, uid, "en", &pool)
                    .await
                    .is_ok()
                {
                    break 'outer;
                }

                let character = entry.item_id.parse()?;

                let db_warp_standard_character = database::DbWarpStandardCharacter {
                    id,
                    uid,
                    character,
                    name: String::new(),
                    timestamp,
                };

                database::set_warp_standard_character(&db_warp_standard_character, &pool).await?;
            } else if entry.item_type == "Light Cone" {
                if database::get_warp_standard_light_cone_by_id_and_uid(id, uid, "en", &pool)
                    .await
                    .is_ok()
                {
                    break 'outer;
                }

                let light_cone = entry.item_id.parse()?;

                let db_warp_standard_light_cone = database::DbWarpStandardLightCone {
                    id,
                    uid,
                    light_cone,
                    name: String::new(),
                    timestamp,
                };

                database::set_warp_standard_light_cone(&db_warp_standard_light_cone, &pool).await?;
            }
        }

        end_id = Some(
            gacha_log.data.list[gacha_log.data.list.len() - 1]
                .id
                .clone(),
        );
    }

    end_id = None;

    //Departure Warp
    'outer: loop {
        let gacha_log: GachaLog = reqwest::get(format!(
            "{url}{}&gacha_type=2",
            end_id.map(|id| format!("&end_id={id}")).unwrap_or_default()
        ))
        .await?
        .json()
        .await?;

        if gacha_log.data.list.is_empty() {
            break;
        }

        for entry in gacha_log.data.list.iter() {
            let id = entry.id.parse()?;
            let uid = entry.uid.parse()?;

            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?;

            if entry.item_type == "Character" {
                if database::get_warp_departure_character_by_id_and_uid(id, uid, "en", &pool)
                    .await
                    .is_ok()
                {
                    break 'outer;
                }

                let character = entry.item_id.parse()?;

                let db_warp_departure_character = database::DbWarpDepartureCharacter {
                    id,
                    uid,
                    character,
                    name: String::new(),
                    timestamp,
                };

                database::set_warp_departure_character(&db_warp_departure_character, &pool).await?;
            } else if entry.item_type == "Light Cone" {
                if database::get_warp_departure_light_cone_by_id_and_uid(id, uid, "en", &pool)
                    .await
                    .is_ok()
                {
                    break 'outer;
                }

                let light_cone = entry.item_id.parse()?;

                let db_warp_departure_light_cone = database::DbWarpDepartureLightCone {
                    id,
                    uid,
                    light_cone,
                    name: String::new(),
                    timestamp,
                };

                database::set_warp_departure_light_cone(&db_warp_departure_light_cone, &pool)
                    .await?;
            }
        }

        end_id = Some(
            gacha_log.data.list[gacha_log.data.list.len() - 1]
                .id
                .clone(),
        );
    }

    end_id = None;

    //Special Warp
    'outer: loop {
        let gacha_log: GachaLog = reqwest::get(format!(
            "{url}{}&gacha_type=11",
            end_id.map(|id| format!("&end_id={id}")).unwrap_or_default()
        ))
        .await?
        .json()
        .await?;

        if gacha_log.data.list.is_empty() {
            break;
        }

        for entry in gacha_log.data.list.iter() {
            let id = entry.id.parse()?;
            let uid = entry.uid.parse()?;

            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?;

            if entry.item_type == "Character" {
                if database::get_warp_special_character_by_id_and_uid(id, uid, "en", &pool)
                    .await
                    .is_ok()
                {
                    break 'outer;
                }

                let character = entry.item_id.parse()?;

                let db_warp_special_character = database::DbWarpSpecialCharacter {
                    id,
                    uid,
                    character,
                    name: String::new(),
                    timestamp,
                };

                database::set_warp_special_character(&db_warp_special_character, &pool).await?;
            } else if entry.item_type == "Light Cone" {
                if database::get_warp_special_light_cone_by_id_and_uid(id, uid, "en", &pool)
                    .await
                    .is_ok()
                {
                    break 'outer;
                }

                let light_cone = entry.item_id.parse()?;

                let db_warp_special_light_cone = database::DbWarpSpecialLightCone {
                    id,
                    uid,
                    light_cone,
                    name: String::new(),
                    timestamp,
                };

                database::set_warp_special_light_cone(&db_warp_special_light_cone, &pool).await?;
            }
        }

        end_id = Some(
            gacha_log.data.list[gacha_log.data.list.len() - 1]
                .id
                .clone(),
        );
    }

    end_id = None;

    //Lc Warp
    'outer: loop {
        let gacha_log: GachaLog = reqwest::get(format!(
            "{url}{}&gacha_type=12",
            end_id.map(|id| format!("&end_id={id}")).unwrap_or_default()
        ))
        .await?
        .json()
        .await?;

        if gacha_log.data.list.is_empty() {
            break;
        }

        for entry in gacha_log.data.list.iter() {
            let id = entry.id.parse()?;
            let uid = entry.uid.parse()?;

            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?;

            if entry.item_type == "Character" {
                if database::get_warp_lc_character_by_id_and_uid(id, uid, "en", &pool)
                    .await
                    .is_ok()
                {
                    break 'outer;
                }

                let character = entry.item_id.parse()?;

                let db_warp_lc_character = database::DbWarpLcCharacter {
                    id,
                    uid,
                    character,
                    name: String::new(),
                    timestamp,
                };

                database::set_warp_lc_character(&db_warp_lc_character, &pool).await?;
            } else if entry.item_type == "Light Cone" {
                if database::get_warp_lc_light_cone_by_id_and_uid(id, uid, "en", &pool)
                    .await
                    .is_ok()
                {
                    break 'outer;
                }

                let light_cone = entry.item_id.parse()?;

                let db_warp_lc_light_cone = database::DbWarpLcLightCone {
                    id,
                    uid,
                    light_cone,
                    name: String::new(),
                    timestamp,
                };

                database::set_warp_lc_light_cone(&db_warp_lc_light_cone, &pool).await?;
            }
        }

        end_id = Some(
            gacha_log.data.list[gacha_log.data.list.len() - 1]
                .id
                .clone(),
        );
    }

    Ok(HttpResponse::Ok().finish())
}
