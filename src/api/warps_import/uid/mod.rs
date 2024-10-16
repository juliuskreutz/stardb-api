use actix_web::{get, web, HttpResponse, Responder};
use utoipa::OpenApi;

use crate::api::{
    warps_import::{WarpsImportInfo, WarpsImportInfos},
    ApiResult,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "warps-import/{uid}")),
    paths(get_warps_import)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_warps_import);
}

#[utoipa::path(
    tag = "warps-import/{uid}",
    get,
    path = "/api/warps-import/{uid}",
    responses(
        (status = 200, description = "WarpsImportInfo", body = WarpsImportInfo)
    )
)]
#[get("/api/warps-import/{uid}")]
async fn get_warps_import(
    uid: web::Path<i32>,
    warps_import_infos: web::Data<WarpsImportInfos>,
) -> ApiResult<impl Responder> {
    let Some(info) = warps_import_infos.lock().await.get(&*uid).cloned() else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let info = info.lock().await.clone();

    Ok(HttpResponse::Ok().json(info))
}
