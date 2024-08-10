use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "stats")),
    paths(get_stats),
    components(schemas(Stats))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_stats);
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct Stats {
    emails: i64,
    hsr_achievement_users: i64,
    zzz_achievement_users: i64,
    gi_achievement_users: i64,
    warp_users: i64,
    signal_users: i64,
    wish_users: i64,
}

#[utoipa::path(
    tag = "stats",
    get,
    path = "/api/stats",
    responses(
        (status = 200, description = "Stats", body = Stats),
    ),
    security(("admin" = []))
)]
#[get("/api/stats")]
async fn get_stats(session: Session, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !database::admins::exists(&username, &pool).await? {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let emails = database::users::count_emails(&pool).await?;
    let hsr_achievement_users =
        database::users_achievements_completed::count_users(100, &pool).await?;
    let zzz_achievement_users =
        database::zzz::users_achievements_completed::count_users(100, &pool).await?;
    let gi_achievement_users =
        database::gi::users_achievements_completed::count_users(100, &pool).await?;
    let warp_users = database::warps::count_uids(&pool).await?;
    let signal_users = database::zzz::signals::count_uids(&pool).await?;
    let wish_users = database::gi::wishes::count_uids(&pool).await?;

    let stats = Stats {
        emails,
        hsr_achievement_users,
        zzz_achievement_users,
        gi_achievement_users,
        warp_users,
        signal_users,
        wish_users,
    };

    Ok(HttpResponse::Ok().json(stats))
}
