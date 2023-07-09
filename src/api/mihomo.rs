use std::fs::File;

use actix_web::{get, web, HttpResponse, Responder};
use serde_json::Value;

use crate::Result;

#[utoipa::path(
    get,
    path = "/api/mihomo/{uid}",
    responses(
        (status = 200, description = "Cached mihomo json"),
    )
)]
#[get("/api/mihomo/{uid}")]
async fn get_mihomo(uid: web::Path<i64>) -> Result<impl Responder> {
    let json: Value = serde_json::from_reader(File::open(format!("mihomo/{uid}.json"))?)?;

    Ok(HttpResponse::Ok().json(json))
}
