use actix_web::{get, web, HttpResponse, Responder};
use utoipa::OpenApi;

use crate::api::{
    zzz::signals_import::{SignalsImportInfo, SignalsImportInfos},
    ApiResult,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "zzz/signals-import/{uid}")),
    paths(get_zzz_signals_import)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_zzz_signals_import);
}

#[utoipa::path(
    tag = "zzz/signals-import/{uid}",
    get,
    path = "/api/zzz/signals-import/{uid}",
    responses(
        (status = 200, description = "SignalsImportInfo", body = SignalsImportInfo)
    )
)]
#[get("/api/zzz/signals-import/{uid}")]
async fn get_zzz_signals_import(
    uid: web::Path<i32>,
    signals_import_infos: web::Data<SignalsImportInfos>,
) -> ApiResult<impl Responder> {
    let Some(info) = signals_import_infos.lock().await.get(&*uid).cloned() else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let info = info.lock().await.clone();

    Ok(HttpResponse::Ok().json(info))
}
