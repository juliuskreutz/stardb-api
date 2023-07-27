use actix_web::{get, web, HttpResponse, Responder};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{api::series::Series, database, Result};

#[derive(OpenApi)]
#[openapi(tags((name = "series/{id}")), paths(get_series))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_series);
}

#[utoipa::path(
    tag = "series/{id}",
    get,
    path = "/api/series/{id}",
    responses(
        (status = 200, description = "Series", body = Series),
    )
)]
#[get("/api/series/{id}")]
async fn get_series(id: web::Path<i32>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let series = Series::from(database::get_series_by_id(*id, &pool).await?);

    Ok(HttpResponse::Ok().json(series))
}
