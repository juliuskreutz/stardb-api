use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut bangboos_id = Vec::new();
    let mut bangboos_rarity = Vec::new();

    for buddy in &configs.buddy["CAHPHFENANN"] {
        let id = buddy.id;

        let rarity = configs.item["CAHPHFENANN"]
            .iter()
            .find(|i| i.id == id)
            .map(|i| i.rarity)
            .unwrap_or(0);

        bangboos_id.push(id);
        bangboos_rarity.push(rarity);
    }

    database::zzz::bangboos::set_all(&bangboos_id, &bangboos_rarity, pool).await?;

    Ok(())
}
