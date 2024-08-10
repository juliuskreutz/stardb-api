use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use super::Leaderboard;
use crate::{
    api::{private, ApiResult, Region},
    database, mihomo, Language,
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

    // Wacky way to update the database in case the uid isn't in there
    if !database::mihomo::exists(uid, &pool).await?
        && mihomo::get(uid, Language::En, &pool).await.is_err()
    {
        let region = match uid.to_string().chars().next() {
            Some('6') => "na",
            Some('7') => "eu",
            Some('8') | Some('9') => "asia",
            _ => "cn",
        }
        .to_string();

        let db_mihomo = database::mihomo::DbMihomo {
            uid,
            region,
            ..Default::default()
        };

        database::mihomo::set(&db_mihomo, &pool).await?;
    }

    let score = database::achievement_scores::get_by_uid(uid, &pool)
        .await?
        .into();

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
