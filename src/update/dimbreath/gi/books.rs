use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut books_id = Vec::new();
    let mut books_series = Vec::new();
    let mut books_series_inside = Vec::new();
    let mut books_icon = Vec::new();

    for localbook_config in configs.localbook_config.values() {
        let id = localbook_config.id;
        let series = localbook_config.series;
        let series_inside = localbook_config.series_inside;

        let icon: i32 = configs
            .item_config
            .get(&id.to_string())
            .or_else(|| configs.item_config_book.get(&id.to_string()))
            .map(|ic| {
                ic.icon_path
                    .strip_prefix("SpriteOutput/ItemIcon/")
                    .unwrap()
                    .strip_suffix(".png")
                    .unwrap()
                    .parse()
                    .unwrap()
            })
            .unwrap_or_default();

        books_id.push(id);
        books_series.push(series);
        books_series_inside.push(series_inside);
        books_icon.push(icon);
    }

    database::set_all_books(
        &books_id,
        &books_series,
        &books_series_inside,
        &books_icon,
        pool,
    )
    .await?;

    Ok(())
}
