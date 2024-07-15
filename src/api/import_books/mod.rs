use std::collections::HashMap;

use actix_multipart::form::MultipartForm;
use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use calamine::{Data, Reader};
use sqlx::PgPool;
use utoipa::OpenApi;

use crate::{
    api::{ApiResult, File},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "import-books")),
    paths(import_books)
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(import_books);
}

#[utoipa::path(
    tag = "pinned",
    post,
    path = "/api/import-books",
    request_body(content = File, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "Successfully imported"),
        (status = 400, description = "Not logged in"),
        (status = 403, description = "Not an admin")
    )
)]
#[post("/api/import-books")]
async fn import_books(
    session: Session,
    file: MultipartForm<File>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if database::admins::get_one_by_username(&username, &pool)
        .await
        .is_err()
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let mut workbook = calamine::open_workbook_auto(&file.file.file)?;

    let mut images = HashMap::new();
    if let Ok(r) = workbook.worksheet_formula("Bookshelf") {
        let (sy, _) = r.start().unwrap_or_default();

        let height = r.height();

        for y in 0..height {
            let image1 = r.get((y, 0)).unwrap().clone();
            let image2 = r.get((y, 6)).unwrap().clone();

            if !image1.is_empty() && !image2.is_empty() {
                let image1 = image1[7..image1.len() - 2].to_string();
                let image2 = image2[7..image2.len() - 2].to_string();

                images.insert(sy as usize + y, (image1, image2));
            }
        }
    }

    for (y, row) in workbook
        .worksheet_range("Bookshelf")?
        .rows()
        .enumerate()
        .skip(1)
        .filter(|(_, data)| data[7] != Data::Empty)
    {
        let ids = match &row[7] {
            &Data::Int(id) => vec![id as i32],
            &Data::Float(id) => vec![id as i32],
            Data::String(ids) => ids.lines().map(|id| id.parse().unwrap()).collect(),
            _ => return Err(anyhow::anyhow!("Id in row {y} is in a wrong format").into()),
        };

        let comment = match &row[5] {
            Data::String(s) => s,
            Data::Empty => "",
            _ => {
                return Err(anyhow::anyhow!("How to obtain in row {y} is in a wrong format").into())
            }
        };

        for id in ids {
            if !comment.is_empty() {
                database::update_book_comment(id, comment, &pool).await?;
            } else {
                database::delete_book_comment(id, &pool).await?;
            }

            if let Some((image1, image2)) = images.get(&(y + 1)) {
                database::update_book_image1(id, image1, &pool).await?;
                database::update_book_image2(id, image2, &pool).await?;
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}
