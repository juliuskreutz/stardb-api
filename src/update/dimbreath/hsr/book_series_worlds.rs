use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut book_series_worlds_id = Vec::new();

    for book_series_world in configs.book_series_world.values() {
        let id = book_series_world.id;

        book_series_worlds_id.push(id);
    }

    database::set_all_book_series_worlds(&book_series_worlds_id, pool).await?;

    Ok(())
}
