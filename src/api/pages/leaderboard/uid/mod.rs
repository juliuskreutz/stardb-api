use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use super::Leaderboard;
use crate::{
    api::{private, ApiResult, Region},
    database,
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
    uid: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let count_na =
        database::count_scores_achievement(Some(&Region::Na.to_string()), None, &pool).await?;
    let count_eu =
        database::count_scores_achievement(Some(&Region::Eu.to_string()), None, &pool).await?;
    let count_asia =
        database::count_scores_achievement(Some(&Region::Asia.to_string()), None, &pool).await?;
    let count_cn =
        database::count_scores_achievement(Some(&Region::Cn.to_string()), None, &pool).await?;
    let count_query = 1;

    let count = count_na + count_eu + count_asia + count_cn;

    let db_score = database::get_score_achievement_by_uid(*uid, &pool)
        .await?
        .into();

    let scores = vec![db_score];

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
