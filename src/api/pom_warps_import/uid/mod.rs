use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::{NaiveDateTime, TimeDelta};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, mihomo, GachaType, Language};

#[derive(OpenApi)]
#[openapi(
    tags((name = "pom-warps-import/{uid}")),
    paths(post_pom_warps_import),
    components(schemas(PomWarpsImportParams)),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(post_pom_warps_import);
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
struct PomWarpsImportParams {
    data: String,
}

#[derive(serde::Deserialize)]
struct Pom {
    default: Default,
}

#[derive(serde::Deserialize)]
struct Default {
    #[serde(rename = "standard")]
    standard: Vec<Warp>,
    #[serde(rename = "beginner")]
    departure: Vec<Warp>,
    #[serde(rename = "character")]
    special: Vec<Warp>,
    #[serde(rename = "lightcone")]
    lc: Vec<Warp>,
}

#[derive(serde::Deserialize)]
struct Warp {
    id: String,
    #[serde(rename = "itemId")]
    item_id: String,
    time: String,
}

#[utoipa::path(
    tag = "pom-warps-import/{uid}",
    post,
    path = "/api/pom-warps-import/{uid}",
    request_body = PomWarpsImportParams,
    responses(
        (status = 200, description = "Warps imported"),
        (status = 403, description = "Not an admin"),
    ),
    security(("admin" = []))
)]
#[post("/api/pom-warps-import/{uid}")]
async fn post_pom_warps_import(
    session: Session,
    uid: web::Path<i32>,
    params: web::Json<PomWarpsImportParams>,
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

    // Wacky way to update the database in case the uid isn't in there
    mihomo::get(*uid, Language::En, &pool).await?;

    let timestamp_offset = chrono::Duration::hours(match uid.to_string().chars().next() {
        Some('6') => -5,
        Some('7') => 1,
        _ => 8,
    });

    let pom: Pom = serde_json::from_str(&params.data)?;

    import_warps(
        *uid,
        timestamp_offset,
        &pom.default.standard,
        GachaType::Standard,
        &pool,
    )
    .await?;
    import_warps(
        *uid,
        timestamp_offset,
        &pom.default.departure,
        GachaType::Departure,
        &pool,
    )
    .await?;
    import_warps(
        *uid,
        timestamp_offset,
        &pom.default.special,
        GachaType::Special,
        &pool,
    )
    .await?;
    import_warps(
        *uid,
        timestamp_offset,
        &pom.default.lc,
        GachaType::Lc,
        &pool,
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
}

async fn import_warps(
    uid: i32,
    timestamp_offset: TimeDelta,
    warps: &[Warp],
    gacha_type: GachaType,
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    for warp in warps {
        let id = warp.id.parse::<i64>().unwrap();
        let item_id = warp.item_id.parse().unwrap();
        let (character, light_cone) = if item_id < 2000 {
            (Some(item_id), None)
        } else {
            (None, Some(item_id))
        };

        let timestamp = NaiveDateTime::parse_from_str(&warp.time, "%Y-%m-%d %H:%M:%S")?.and_utc()
            - timestamp_offset;

        // FIXME: Temp
        database::delete_warp_by_id_and_timestamp(id, timestamp + chrono::Duration::hours(5), pool)
            .await?;
        database::delete_warp_by_id_and_timestamp(id, timestamp + chrono::Duration::hours(6), pool)
            .await?;
        database::delete_warp_by_id_and_timestamp(
            id,
            timestamp + chrono::Duration::hours(13),
            pool,
        )
        .await?;
        // FIXME: Temp

        let warp = database::DbWarp {
            id,
            uid,
            gacha_type,
            character,
            light_cone,
            name: None,
            rarity: None,
            timestamp,
            official: false,
        };

        database::set_warp(&warp, pool).await?;
    }

    Ok(())
}
