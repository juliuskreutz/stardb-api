use std::time::{Duration, Instant};

use anyhow::Result;
use sqlx::PgPool;

use crate::database;

pub async fn books_percent(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5 * 60));

        loop {
            interval.tick().await;

            let start = Instant::now();

            if let Err(e) = update(&pool).await {
                log::error!(
                    "Books Percent update failed with {e} in {}s",
                    start.elapsed().as_secs_f64()
                );
            } else {
                log::info!(
                    "Books Percent update succeeded in {}s",
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
}

async fn update(pool: &PgPool) -> Result<()> {
    let total_count = database::get_users_books_user_count(pool).await? as f64;

    let books_users_count = database::get_books_users_count(pool).await?;

    for book_users_count in books_users_count {
        let id = book_users_count.id;
        let percent = book_users_count.count.unwrap_or_default() as f64 / total_count;

        let book_percent = database::DbBookPercent { id, percent };

        database::set_book_percent(&book_percent, pool).await?;
    }

    Ok(())
}
