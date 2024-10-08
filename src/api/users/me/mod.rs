mod achievements;
mod email;
mod export;
mod gi;
mod import;
mod password;
mod uids;
mod username;
mod zzz;

use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me")),
    paths(get_me),
    components(schemas(
        Uid,
        User,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievements::openapi());
    openapi.merge(email::openapi());
    openapi.merge(export::openapi());
    openapi.merge(gi::openapi());
    openapi.merge(import::openapi());
    openapi.merge(password::openapi());
    openapi.merge(uids::openapi());
    openapi.merge(username::openapi());
    openapi.merge(zzz::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_me)
        .configure(achievements::configure)
        .configure(email::configure)
        .configure(export::configure)
        .configure(gi::configure)
        .configure(import::configure)
        .configure(password::configure)
        .configure(uids::configure)
        .configure(username::configure)
        .configure(zzz::configure);
}

#[derive(Serialize, ToSchema)]
pub struct Uid {
    uid: i32,
    verified: bool,
    private: bool,
}

#[derive(Serialize, ToSchema)]
pub struct User {
    username: String,
    admin: bool,
    email: Option<String>,
    uids: Vec<Uid>,
    zzz_uids: Vec<Uid>,
    gi_uids: Vec<Uid>,
    achievements: Vec<i32>,
    zzz_achievements: Vec<i32>,
}

#[utoipa::path(
    tag = "users/me",
    get,
    path = "/api/users/me",
    responses(
        (status = 200, description = "User", body = User),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/users/me")]
async fn get_me(session: Session, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    //session.renew();

    let admin = database::admins::exists(&username, &pool).await?;

    let user = database::users::get_one_by_username(&username, &pool).await?;

    let email = user.email;

    let uids = database::connections::get_by_username(&username, &pool)
        .await?
        .into_iter()
        .map(|c| Uid {
            uid: c.uid,
            verified: c.verified,
            private: c.private,
        })
        .collect();

    let zzz_uids = database::zzz::connections::get_by_username(&username, &pool)
        .await?
        .into_iter()
        .map(|c| Uid {
            uid: c.uid,
            verified: c.verified,
            private: c.private,
        })
        .collect();

    let gi_uids = database::gi::connections::get_by_username(&username, &pool)
        .await?
        .into_iter()
        .map(|c| Uid {
            uid: c.uid,
            verified: c.verified,
            private: c.private,
        })
        .collect();

    let achievements = database::users_achievements_completed::get_by_username(&username, &pool)
        .await?
        .into_iter()
        .map(|b| b.id)
        .collect();

    let zzz_achievements =
        database::zzz::users_achievements_completed::get_by_username(&username, &pool)
            .await?
            .into_iter()
            .map(|b| b.id)
            .collect();

    let user = User {
        username,
        admin,
        email,
        uids,
        zzz_uids,
        gi_uids,
        achievements,
        zzz_achievements,
    };

    Ok(HttpResponse::Ok().json(user))
}
