use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use actix_web::rt;
use anyhow::Result;
use sqlx::PgPool;

use crate::database;

pub async fn spawn(pool: PgPool) {
    rt::spawn(async move {
        let mut interval = rt::time::interval(Duration::from_secs(5 * 60));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(&pool).await {
                error!(
                    "Books Percent update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                info!(
                    "Books Percent update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update(pool: &PgPool) -> Result<()> {
    let total_count = database::get_users_books_completed_user_count(pool).await? as f64;

    let books_users_count = database::get_books_users_count(pool).await?;

    let mut books_users_count_map = HashMap::new();

    for id in database::get_books_id(pool).await? {
        books_users_count_map.insert(id, 0.0);
    }

    for book_users_count in books_users_count {
        let id = book_users_count.id;
        let percent = book_users_count.count.unwrap_or_default() as f64 / total_count;

        books_users_count_map.insert(id, percent);
    }

    for (id, percent) in books_users_count_map {
        let book_percent = database::DbBookPercent { id, percent };

        database::set_book_percent(&book_percent, pool).await?;
    }

    Ok(())
}
