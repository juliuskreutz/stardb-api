use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps-stats/{uid}")),
    paths(get_warps_stats),
    components(schemas(WarpsStats, WarpsStatsGachaType, WarpsStatsTotal))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct WarpsStats {
    departure_4: Option<WarpsStatsGachaType>,
    standard_4: Option<WarpsStatsGachaType>,
    special_4: Option<WarpsStatsGachaType>,
    lc_4: Option<WarpsStatsGachaType>,
    departure_5: Option<WarpsStatsGachaType>,
    standard_5: Option<WarpsStatsGachaType>,
    special_5: Option<WarpsStatsGachaType>,
    lc_5: Option<WarpsStatsGachaType>,
    total: WarpsStatsTotal,
}

#[derive(Serialize, ToSchema)]
struct WarpsStatsGachaType {
    count: i64,
    avg: f64,
    median: i64,
    sum: i64,
    rank_count: i64,
    rank_avg: i64,
    rank_median: i64,
    rank_sum: i64,
    total: i64,
}

#[derive(Serialize, ToSchema)]
struct WarpsStatsTotal {
    sum: i64,
    rank: i64,
    total: i64,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_warps_stats);
}

#[utoipa::path(
    tag = "warps-stats/{uid}",
    get,
    path = "/api/warps-stats/{uid}",
    responses(
        (status = 200, description = "WarpsStats", body = WarpsStats),
    )
)]
#[get("/api/warps-stats/{uid}")]
async fn get_warps_stats(
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let warps_stats_gacha_type_4 = database::get_warps_stats_4_by_uid(*uid, &pool).await?;
    let warps_stats_gacha_type_5 = database::get_warps_stats_5_by_uid(*uid, &pool).await?;
    let warps_stats_gacha_type = database::get_warps_stats_gacha_type(&pool).await?;

    let warps_stats_uid = database::get_warps_stats_by_uid(*uid, &pool).await?;
    let warps_stats = database::get_warps_stats(&pool).await?;

    let mut departure_4 = None;
    let mut standard_4 = None;
    let mut special_4 = None;
    let mut lc_4 = None;

    for warps_stats in warps_stats_gacha_type_4 {
        match warps_stats.gacha_type.as_str() {
            "departure" => departure_4 = Some(warps_stats),
            "standard" => standard_4 = Some(warps_stats),
            "special" => special_4 = Some(warps_stats),
            "lc" => lc_4 = Some(warps_stats),
            _ => {}
        }
    }

    let mut departure_5 = None;
    let mut standard_5 = None;
    let mut special_5 = None;
    let mut lc_5 = None;

    for warps_stats in warps_stats_gacha_type_5 {
        match warps_stats.gacha_type.as_str() {
            "departure" => departure_5 = Some(warps_stats),
            "standard" => standard_5 = Some(warps_stats),
            "special" => special_5 = Some(warps_stats),
            "lc" => lc_5 = Some(warps_stats),
            _ => {}
        }
    }

    let mut departure_total = None;
    let mut standard_total = None;
    let mut special_total = None;
    let mut lc_total = None;

    for warps_stats in warps_stats_gacha_type {
        match warps_stats.gacha_type.as_str() {
            "departure" => departure_total = Some(warps_stats.total.unwrap_or(0)),
            "standard" => standard_total = Some(warps_stats.total.unwrap_or(0)),
            "special" => special_total = Some(warps_stats.total.unwrap_or(0)),
            "lc" => lc_total = Some(warps_stats.total.unwrap_or(0)),
            _ => {}
        }
    }

    let departure_4 = departure_4.map(|warps_stats| WarpsStatsGachaType {
        count: warps_stats.count,
        avg: warps_stats.avg,
        median: warps_stats.median,
        sum: warps_stats.sum,
        rank_count: warps_stats.rank_count,
        rank_avg: warps_stats.rank_avg,
        rank_median: warps_stats.rank_median,
        rank_sum: warps_stats.rank_sum,
        total: departure_total.unwrap_or(0),
    });
    let standard_4 = standard_4.map(|warps_stats| WarpsStatsGachaType {
        count: warps_stats.count,
        avg: warps_stats.avg,
        median: warps_stats.median,
        sum: warps_stats.sum,
        rank_count: warps_stats.rank_count,
        rank_avg: warps_stats.rank_avg,
        rank_median: warps_stats.rank_median,
        rank_sum: warps_stats.rank_sum,
        total: standard_total.unwrap_or(0),
    });
    let special_4 = special_4.map(|warps_stats| WarpsStatsGachaType {
        count: warps_stats.count,
        avg: warps_stats.avg,
        median: warps_stats.median,
        sum: warps_stats.sum,
        rank_count: warps_stats.rank_count,
        rank_avg: warps_stats.rank_avg,
        rank_median: warps_stats.rank_median,
        rank_sum: warps_stats.rank_sum,
        total: special_total.unwrap_or(0),
    });
    let lc_4 = lc_4.map(|warps_stats| WarpsStatsGachaType {
        count: warps_stats.count,
        avg: warps_stats.avg,
        median: warps_stats.median,
        sum: warps_stats.sum,
        rank_count: warps_stats.rank_count,
        rank_avg: warps_stats.rank_avg,
        rank_median: warps_stats.rank_median,
        rank_sum: warps_stats.rank_sum,
        total: lc_total.unwrap_or(0),
    });

    let departure_5 = departure_5.map(|warps_stats| WarpsStatsGachaType {
        count: warps_stats.count,
        avg: warps_stats.avg,
        median: warps_stats.median,
        sum: warps_stats.sum,
        rank_count: warps_stats.rank_count,
        rank_avg: warps_stats.rank_avg,
        rank_median: warps_stats.rank_median,
        rank_sum: warps_stats.rank_sum,
        total: departure_total.unwrap_or(0),
    });
    let standard_5 = standard_5.map(|warps_stats| WarpsStatsGachaType {
        count: warps_stats.count,
        avg: warps_stats.avg,
        median: warps_stats.median,
        sum: warps_stats.sum,
        rank_count: warps_stats.rank_count,
        rank_avg: warps_stats.rank_avg,
        rank_median: warps_stats.rank_median,
        rank_sum: warps_stats.rank_sum,
        total: standard_total.unwrap_or(0),
    });
    let special_5 = special_5.map(|warps_stats| WarpsStatsGachaType {
        count: warps_stats.count,
        avg: warps_stats.avg,
        median: warps_stats.median,
        sum: warps_stats.sum,
        rank_count: warps_stats.rank_count,
        rank_avg: warps_stats.rank_avg,
        rank_median: warps_stats.rank_median,
        rank_sum: warps_stats.rank_sum,
        total: special_total.unwrap_or(0),
    });
    let lc_5 = lc_5.map(|warps_stats| WarpsStatsGachaType {
        count: warps_stats.count,
        avg: warps_stats.avg,
        median: warps_stats.median,
        sum: warps_stats.sum,
        rank_count: warps_stats.rank_count,
        rank_avg: warps_stats.rank_avg,
        rank_median: warps_stats.rank_median,
        rank_sum: warps_stats.rank_sum,
        total: lc_total.unwrap_or(0),
    });

    let total = WarpsStatsTotal {
        sum: warps_stats_uid.sum.unwrap_or(0),
        rank: warps_stats_uid.rank.unwrap_or(0),
        total: warps_stats.total.unwrap_or(0),
    };

    let warps_stats = WarpsStats {
        departure_4,
        standard_4,
        special_4,
        lc_4,
        departure_5,
        standard_5,
        special_5,
        lc_5,
        total,
    };

    Ok(HttpResponse::Ok().json(warps_stats))
}
