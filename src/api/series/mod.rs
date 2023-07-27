mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::{self, DbSeries},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "series")),
    paths(get_series),
    components(schemas(
        Series
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct Series {
    id: i32,
    name: String,
}

impl From<DbSeries> for Series {
    fn from(db_series: DbSeries) -> Self {
        Self {
            id: db_series.id,
            name: db_series.name,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_series).configure(id::configure);
}

#[utoipa::path(
    tag = "series",
    get,
    path = "/api/series",
    responses(
        (status = 200, description = "[Series]", body = Vec<Series>),
    )
)]
#[get("/api/series")]
async fn get_series(pool: web::Data<PgPool>) -> Result<impl Responder> {
    let series: Vec<_> = database::get_series(&pool)
        .await?
        .into_iter()
        .map(Series::from)
        .collect();

    Ok(HttpResponse::Ok().json(series))
}
