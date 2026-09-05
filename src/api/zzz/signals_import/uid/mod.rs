use actix_web::{get, web, HttpResponse, Responder};
use utoipa::OpenApi;

use crate::api::{
    import_jobs::ImportJobId,
    zzz::signals_import::{SignalsImportInfo, SignalsImportInfos},
    ApiResult,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "zzz/signals-import/jobs/{job_id}")),
    paths(get_zzz_signals_import)
)]
struct ApiDoc;

/// Returns this module's OpenAPI definition.
pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

/// Registers this module's routes and any shared application data.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_zzz_signals_import);
}

#[utoipa::path(
    tag = "zzz/signals-import/jobs/{job_id}",
    get,
    path = "/api/zzz/signals-import/jobs/{job_id}",
    params(("job_id" = String, Path)),
    responses(
        (status = 200, description = "SignalsImportInfo", body = SignalsImportInfo)
    )
)]
#[get("/api/zzz/signals-import/jobs/{job_id}")]
/// Returns a snapshot for an opaque ZZZ job ID, or 400 when absent or evicted.
async fn get_zzz_signals_import(
    job_id: web::Path<ImportJobId>,
    signals_import_infos: web::Data<SignalsImportInfos>,
) -> ApiResult<impl Responder> {
    let Some(info) = signals_import_infos.get(*job_id).await else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let info = info.lock().await.clone();

    Ok(HttpResponse::Ok().json(info))
}
