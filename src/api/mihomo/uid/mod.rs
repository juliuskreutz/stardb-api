use actix_web::{get, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{ApiResult, LanguageParams},
    mihomo,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "mihomo/{uid}")),
    paths(get_mihomo, put_mihomo)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_mihomo).service(put_mihomo);
}

#[utoipa::path(
    tag = "mihomo/{uid}",
    get,
    path = "/api/mihomo/{uid}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Cached mihomo json"),
    )
)]
#[get("/api/mihomo/{uid}")]
async fn get_mihomo(
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let json = mihomo::get(*uid, language_params.lang, &pool).await?;

    Ok(HttpResponse::Ok().json(json))
}

#[utoipa::path(
    tag = "mihomo/{uid}",
    put,
    path = "/api/mihomo/{uid}",
    params(LanguageParams),
    responses(
        (status = 200, description = "Updated"),
    )
)]
#[put("/api/mihomo/{uid}")]
async fn put_mihomo(
    uid: web::Path<i32>,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let uid = *uid;

    let json = mihomo::update_and_get(uid, language_params.lang, &pool).await?;

    Ok(HttpResponse::Ok().json(json))
}
