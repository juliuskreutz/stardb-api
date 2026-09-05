use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use super::Leaderboard;
use crate::{
    api::{private, ApiResult, Region},
    database, mihomo,
};

#[derive(OpenApi)]
#[openapi(paths(get_leaderboard_entry))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_leaderboard_entry);
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/leaderboard/{uid}",
    security(("api_key" = [])),
    responses(
        (status = 200, description = "Leaderboard"),
    )
)]
#[get("/api/pages/leaderboard/{uid}", guard = "private")]
async fn get_leaderboard_entry(
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    if !(100000000..1000000000).contains(&uid) {
        return Ok(HttpResponse::BadRequest().finish());
    }

    mihomo::ensure_row(uid, &pool).await?;

    let Some(score) = database::achievement_scores::get_by_uid(uid, &pool).await? else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let score = score.into();

    let count_na =
        database::achievement_scores::count(Some(&Region::Na.to_string()), None, &pool).await?;
    let count_eu =
        database::achievement_scores::count(Some(&Region::Eu.to_string()), None, &pool).await?;
    let count_asia =
        database::achievement_scores::count(Some(&Region::Asia.to_string()), None, &pool).await?;
    let count_cn =
        database::achievement_scores::count(Some(&Region::Cn.to_string()), None, &pool).await?;
    let count_query = 1;

    let count = count_na + count_eu + count_asia + count_cn;

    let scores = vec![score];

    let leaderboard = Leaderboard {
        count,
        count_na,
        count_eu,
        count_asia,
        count_cn,
        count_query,
        scores,
    };

    Ok(HttpResponse::Ok().json(leaderboard))
}
