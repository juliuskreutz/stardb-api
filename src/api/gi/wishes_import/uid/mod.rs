use actix_web::{get, web, HttpResponse, Responder};
use utoipa::OpenApi;

use crate::api::{
    gi::wishes_import::{WishesImportInfo, WishesImportInfos},
    ApiResult,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/wishes-import/{uid}")),
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
    tag = "gi/wishes-import/{uid}",
    get,
    path = "/api/gi/wishes-import/{uid}",
    responses(
        (status = 200, description = "WishesImportInfo", body = WishesImportInfo)
    )
)]
#[get("/api/gi/wishes-import/{uid}")]
async fn get_gi_wishes_import(
    uid: web::Path<i32>,
    wishes_import_infos: web::Data<WishesImportInfos>,
) -> ApiResult<impl Responder> {
    let Some(info) = wishes_import_infos.lock().await.get(&*uid).cloned() else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let info = info.lock().await.clone();

    Ok(HttpResponse::Ok().json(info))
}
