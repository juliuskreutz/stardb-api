mod uid;

use actix_web::{post, web, HttpResponse, Responder};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::Display;
use url::Url;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps")),
    paths(post_warps),
    components(schemas(WarpParams, WarpImport, GachaType))
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
struct WarpParams {
    url: String,
    gacha_type: GachaType,
    end_id: Option<String>,
}

#[derive(Serialize, ToSchema)]
struct WarpImport {
    count: usize,
    uid: Option<i64>,
    end_id: Option<String>,
}

#[utoipa::path(
    tag = "warps",
    post,
    path = "/api/warps",
    request_body = WarpImport,
    responses(
        (status = 200, description = "WarpImport", body = WarpImport),
    )
)]
#[post("/api/warps")]
async fn post_warps(
    params: web::Json<WarpParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let url = Url::parse(&params.url)?;

    let query = url.query_pairs().filter(|(name, _)| {
        matches!(
            name.to_string().as_str(),
            "authkey" | "authkey_ver" | "sign_type"
        )
    });

    let mut url =
        Url::parse("https://api-os-takumi.mihoyo.com/common/gacha_record/api/getGachaLog")?;

    url.query_pairs_mut()
        .extend_pairs(query)
        .extend_pairs(&[
            ("lang", "en"),
            ("game_biz", "hkrpg_global"),
            ("size", "20"),
            (
                "gacha_type",
                match params.gacha_type {
                    GachaType::Standard => "1",
                    GachaType::Departure => "2",
                    GachaType::Special => "11",
                    GachaType::Lc => "12",
                },
            ),
            ("end_id", params.end_id.as_deref().unwrap_or("0")),
        ])
        .finish();

    let gacha_log: GachaLog = reqwest::get(url).await?.json().await?;

    let mut warp_import = WarpImport {
        count: 0,
        uid: None,
        end_id: None,
    };

    for entry in gacha_log.data.list.iter() {
        warp_import.end_id = Some(entry.id.clone());

        let id = entry.id.parse()?;
        let gacha_type = params.gacha_type.to_string();

        if database::get_warp_by_id_and_gacha_type(id, &gacha_type, "en", &pool)
            .await
            .is_ok()
        {
            continue;
        }

        let uid = entry.uid.parse()?;
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

        warp_import.uid = Some(uid);
        warp_import.count += 1;
    }

    Ok(HttpResponse::Ok().json(warp_import))
}
