use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use actix_web::{get, web, HttpResponse, Responder};
use anyhow::Result;
use async_rwlock::RwLock;
use indexmap::IndexMap;
use serde::Serialize;
use sqlx::PgPool;
use strum::IntoEnumIterator;
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, Language, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(paths(get_book_tracker))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_book_tracker);
}

#[derive(Default)]
pub struct BookTrackerCache {
    book_tracker_map: RwLock<HashMap<Language, BookTracker>>,
}

#[derive(Default, Serialize)]
struct BookTracker {
    book_count: usize,
    user_count: i64,
    language: Language,
    worlds: Vec<World>,
}

#[derive(Serialize)]
struct World {
    world: String,
    book_count: usize,
    series: Vec<Series>,
}

#[derive(Serialize)]
struct Series {
    series: String,
    book_count: usize,
    books: Vec<Book>,
}

#[derive(Serialize)]
struct Book {
    id: i64,
    series: i32,
    series_name: String,
    series_world: i32,
    series_world_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<i32>,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    percent: f64,
}

impl From<database::DbBook> for Book {
    fn from(db_book: database::DbBook) -> Self {
        Book {
            id: db_book.id,
            series: db_book.series,
            series_name: db_book.series_name.clone(),
            series_world: db_book.series_world,
            series_world_name: db_book.series_world_name,
            icon: db_book.icon,
            name: db_book.name.clone(),
            comment: db_book.comment.clone(),
            percent: db_book.percent,
        }
    }
}

pub fn cache(pool: PgPool) -> web::Data<BookTrackerCache> {
    let book_tracker_cache = web::Data::new(BookTrackerCache::default());

    {
        let book_tracker_cache = book_tracker_cache.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let start = Instant::now();

                if let Err(e) = update(&book_tracker_cache, &pool).await {
                    log::error!(
                        "Book Tracker update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    log::info!(
                        "Book Tracker update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });
    }

    book_tracker_cache
}

async fn update(book_tracker_cache: &web::Data<BookTrackerCache>, pool: &PgPool) -> Result<()> {
    let mut book_tracker_map = HashMap::new();

    for language in Language::iter() {
        let db_books = database::get_books(&language.to_string(), pool).await?;

        let mut worlds: IndexMap<String, IndexMap<String, Vec<Book>>> = IndexMap::new();

        for db_book in db_books {
            worlds
                .entry(db_book.series_world_name.clone())
                .or_default()
                .entry(db_book.series_name.clone())
                .or_default()
                .push(Book::from(db_book));
        }

        let worlds = worlds
            .into_iter()
            .map(|(world, series)| {
                let series = series
                    .into_iter()
                    .map(|(series, books)| Series {
                        series,
                        book_count: books.len(),
                        books,
                    })
                    .collect::<Vec<_>>();

                let book_count = series.iter().map(|bs| bs.book_count).sum();

                World {
                    world,
                    book_count,
                    series,
                }
            })
            .collect::<Vec<_>>();

        let book_count = worlds.iter().map(|bw| bw.book_count).sum();
        let user_count = database::get_users_books_user_count(pool).await?;

        let book_tracker = BookTracker {
            book_count,
            user_count,
            language,
            worlds,
        };

        book_tracker_map.insert(language, book_tracker);
    }

    *book_tracker_cache.book_tracker_map.write().await = book_tracker_map;

    Ok(())
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/book-tracker",
    params(LanguageParams),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "BookTracker"),
    )
)]
#[get("/api/pages/book-tracker", guard = "private")]
async fn get_book_tracker(
    language_params: web::Query<LanguageParams>,
    book_tracker_cache: web::Data<BookTrackerCache>,
) -> ApiResult<impl Responder> {
    Ok(HttpResponse::Ok()
        .json(&book_tracker_cache.book_tracker_map.read().await[&language_params.lang]))
}
