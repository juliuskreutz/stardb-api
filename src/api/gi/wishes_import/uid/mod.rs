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

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

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
