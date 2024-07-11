mod private;

use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::ApiResult, database, mihomo, Language};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/me/uids/{uid}")),
    paths(put_user_uid, delete_user_uid),
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(private::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(private::configure)
        .service(put_user_uid)
        .service(delete_user_uid);
}

#[utoipa::path(
    tag = "users/me/uids/{uid}",
    put,
    path = "/api/users/me/uids/{uid}",
    responses(
        (status = 200, description = "Added uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/users/me/uids/{uid}")]
async fn put_user_uid(
    session: Session,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let uid = *uid;

    if !(100000000..1000000000).contains(&uid) {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let connection = database::DbConnection {
        username,
        uid,
        verified: false,
        private: false,
    };

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

    database::set_connection(&connection, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "users/me/uids/{uid}",
    delete,
    path = "/api/users/me/uids/{uid}",
    responses(
        (status = 200, description = "Deleted uid"),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/users/me/uids/{uid}")]
async fn delete_user_uid(
    session: Session,
    uid: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let connection = database::DbConnection {
        username,
        uid: *uid,
        verified: false,
        private: false,
    };

    database::delete_connection(&connection, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
