mod uid;

use actix_web::{post, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::PgPool;
use strum::Display;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps")),
    paths(post_warps),
    components(schemas(WarpParameters, GachaType))
)]
struct ApiDoc;

#[derive(Display, Deserialize, ToSchema)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum GachaType {
    Standard,
    Departure,
    Special,
    Lc,
}

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
struct WarpParameters {
    parameters: String,
}

#[utoipa::path(
    tag = "warps",
    post,
    path = "/api/warps",
    request_body = WarpParameters,
    responses(
        (status = 200, description = "Uid", body = i64),
    )
)]
#[post("/api/warps")]
async fn post_warps(
    parameters: web::Json<WarpParameters>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let parameters: Vec<_> = parameters.parameters.split_whitespace().collect();

    let auth_key = urlencoding::encode(parameters[0]);
    let authkey_ver = parameters[1];
    let sign_type = parameters[2];

    let url = format!("https://api-os-takumi.mihoyo.com/common/gacha_record/api/getGachaLog?auth_key={auth_key}&authkey_ver={authkey_ver}&sign_type={sign_type}&game_biz=hkrpg_global&size=20");

    let mut end_id = None;
    let mut uid = 0;

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
            let gacha_type = GachaType::Standard.to_string();

            if database::get_warp_by_id_and_gacha_type(id, &gacha_type, "en", &pool)
                .await
                .is_ok()
            {
                break 'outer;
            }

            uid = entry.uid.parse()?;

            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?;

            let item = entry.item_id.parse()?;

            if entry.item_type == "Character" {
                let db_warp = database::DbWarp {
                    id,
                    uid,
                    character: Some(item),
                    light_cone: None,
                    gacha_type,
                    name: None,
                    rarity: None,
                    timestamp,
                };

                database::set_warp(&db_warp, &pool).await?;
            } else if entry.item_type == "Light Cone" {
                let db_warp = database::DbWarp {
                    id,
                    uid,
                    character: None,
                    light_cone: Some(item),
                    gacha_type,
                    name: None,
                    rarity: None,
                    timestamp,
                };

                database::set_warp(&db_warp, &pool).await?;
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
            let gacha_type = GachaType::Departure.to_string();

            if database::get_warp_by_id_and_gacha_type(id, &gacha_type, "en", &pool)
                .await
                .is_ok()
            {
                break 'outer;
            }

            uid = entry.uid.parse()?;

            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?;

            let item = entry.item_id.parse()?;

            if entry.item_type == "Character" {
                let db_warp = database::DbWarp {
                    id,
                    uid,
                    character: Some(item),
                    light_cone: None,
                    gacha_type,
                    name: None,
                    rarity: None,
                    timestamp,
                };

                database::set_warp(&db_warp, &pool).await?;
            } else if entry.item_type == "Light Cone" {
                let db_warp = database::DbWarp {
                    id,
                    uid,
                    character: None,
                    light_cone: Some(item),
                    gacha_type,
                    name: None,
                    rarity: None,
                    timestamp,
                };

                database::set_warp(&db_warp, &pool).await?;
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
            let gacha_type = GachaType::Special.to_string();

            if database::get_warp_by_id_and_gacha_type(id, &gacha_type, "en", &pool)
                .await
                .is_ok()
            {
                break 'outer;
            }

            uid = entry.uid.parse()?;

            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?;

            let item = entry.item_id.parse()?;

            if entry.item_type == "Character" {
                let db_warp = database::DbWarp {
                    id,
                    uid,
                    character: Some(item),
                    light_cone: None,
                    gacha_type,
                    name: None,
                    rarity: None,
                    timestamp,
                };

                database::set_warp(&db_warp, &pool).await?;
            } else if entry.item_type == "Light Cone" {
                let db_warp = database::DbWarp {
                    id,
                    uid,
                    character: None,
                    light_cone: Some(item),
                    gacha_type,
                    name: None,
                    rarity: None,
                    timestamp,
                };

                database::set_warp(&db_warp, &pool).await?;
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
            let gacha_type = GachaType::Lc.to_string();

            if database::get_warp_by_id_and_gacha_type(id, &gacha_type, "en", &pool)
                .await
                .is_ok()
            {
                break 'outer;
            }

            uid = entry.uid.parse()?;

            let timestamp = NaiveDateTime::parse_from_str(&entry.time, "%Y-%m-%d %H:%M:%S")?;

            let item = entry.item_id.parse()?;

            if entry.item_type == "Character" {
                let db_warp = database::DbWarp {
                    id,
                    uid,
                    character: Some(item),
                    light_cone: None,
                    gacha_type,
                    name: None,
                    rarity: None,
                    timestamp,
                };

                database::set_warp(&db_warp, &pool).await?;
            } else if entry.item_type == "Light Cone" {
                let db_warp = database::DbWarp {
                    id,
                    uid,
                    character: None,
                    light_cone: Some(item),
                    gacha_type,
                    name: None,
                    rarity: None,
                    timestamp,
                };

                database::set_warp(&db_warp, &pool).await?;
            }
        }

        end_id = Some(
            gacha_log.data.list[gacha_log.data.list.len() - 1]
                .id
                .clone(),
        );
    }

    Ok(HttpResponse::Ok().json(uid))
}
