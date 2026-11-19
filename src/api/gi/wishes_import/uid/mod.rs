use actix_web::{get, web, HttpResponse, Responder};
use utoipa::OpenApi;

use crate::api::{
    gi::wishes_import::{WishesImportInfo, WishesImportInfos},
    import_jobs::ImportJobId,
    ApiResult,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/wishes-import/jobs/{job_id}")),
    paths(get_gi_wishes_import)
)]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module's routes and any shared application data.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_gi_wishes_import);
}

#[utoipa::path(
    tag = "gi/wishes-import/jobs/{job_id}",
    get,
    path = "/api/gi/wishes-import/jobs/{job_id}",
    params(("job_id" = String, Path)),
    responses(
        (status = 200, description = "WishesImportInfo", body = WishesImportInfo)
    )
)]
#[get("/api/gi/wishes-import/jobs/{job_id}")]
/// Returns a snapshot for an opaque GI job ID, or 400 when absent or evicted.
async fn get_gi_wishes_import(
    job_id: web::Path<ImportJobId>,
    wishes_import_infos: web::Data<WishesImportInfos>,
) -> ApiResult<impl Responder> {
    let Some(info) = wishes_import_infos.get(*job_id).await else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let info = info.lock().await.clone();

    Ok(HttpResponse::Ok().json(info))
}
