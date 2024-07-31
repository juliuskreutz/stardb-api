use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut book_series_id = Vec::new();
    let mut book_series_world = Vec::new();
    let mut book_series_bookself = Vec::new();

    for book_series_config in &configs.book_series_config {
        let id = book_series_config.id;
        let world = book_series_config.world;
        let bookshelf = book_series_config.bookshelf.unwrap_or_default();

        book_series_id.push(id);
        book_series_world.push(world);
        book_series_bookself.push(bookshelf);
    }

    database::set_all_book_series(
        &book_series_id,
        &book_series_world,
        &book_series_bookself,
        pool,
    )
    .await?;

    Ok(())
}
