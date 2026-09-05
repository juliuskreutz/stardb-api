use actix_web::{get, web, HttpResponse, Responder};
use utoipa::OpenApi;

use crate::api::{
    import_jobs::ImportJobId,
    warps_import::{WarpsImportInfo, WarpsImportInfos},
    ApiResult,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps-import/jobs/{job_id}")),
    paths(get_warps_import)
)]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module's routes and any shared application data.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_warps_import);
}

#[utoipa::path(
    tag = "warps-import/jobs/{job_id}",
    get,
    path = "/api/warps-import/jobs/{job_id}",
    params(("job_id" = String, Path)),
    responses(
        (status = 200, description = "WarpsImportInfo", body = WarpsImportInfo)
    )
)]
#[get("/api/warps-import/jobs/{job_id}")]
/// Returns a snapshot for an opaque HSR job ID, or 400 when absent or evicted.
async fn get_warps_import(
    job_id: web::Path<ImportJobId>,
    warps_import_infos: web::Data<WarpsImportInfos>,
) -> ApiResult<impl Responder> {
    let Some(info) = warps_import_infos.get(*job_id).await else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let info = info.lock().await.clone();

    Ok(HttpResponse::Ok().json(info))
}
