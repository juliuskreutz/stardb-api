use std::{fs::File, path::PathBuf};

use actix_web::{get, web, HttpResponse, Responder};
use serde_json::Value;
use utoipa::OpenApi;

use crate::Result;

#[derive(OpenApi)]
#[openapi(tags((name = "mihomo/{uid}")), paths(get_mihomo))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_mihomo);
}

#[utoipa::path(
    tag = "mihomo/{uid}",
    get,
    path = "/api/mihomo/{uid}",
    responses(
        (status = 200, description = "Cached mihomo json"),
    )
)]
#[get("/api/mihomo/{uid}")]
async fn get_mihomo(uid: web::Path<i64>) -> Result<impl Responder> {
    let path = format!("mihomo/{uid}.json");

    if !PathBuf::from(&path).exists() {
        reqwest::Client::new()
            .put(&format!(
                "http://localhost:8000/api/scores/achievements/{uid}"
            ))
            .send()
            .await?;
    }

    let json: Value = serde_json::from_reader(File::open(&path)?)?;

    Ok(HttpResponse::Ok().json(json))
}
