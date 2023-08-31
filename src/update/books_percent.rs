use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::Result;
use sqlx::PgPool;

use crate::database;

pub async fn books_percent(pool: PgPool) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

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
    let completed = database::get_users_books(pool).await?;

    let mut usernames_books: HashMap<String, Vec<i64>> = HashMap::new();

    for completed in &completed {
        usernames_books
            .entry(completed.username.clone())
            .or_default()
            .push(completed.id)
    }

    let mut books_count: HashMap<i64, usize> = HashMap::new();

    for books in usernames_books.values().filter(|v| v.len() >= 50) {
        for &book in books {
            *books_count.entry(book).or_default() += 1;
        }
    }

    let books_id = database::get_books_id(pool).await?;

    for id in books_id {
        let percent = if let Some(&count) = books_count.get(&id) {
            count as f64 / usernames_books.len() as f64
        } else {
            0.0
        };

        let book_percent = database::DbBookPercent { id, percent };

        database::set_book_percent(&book_percent, pool).await?;
    }

    Ok(())
}
