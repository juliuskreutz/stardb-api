use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut w_engines_id = Vec::new();
    let mut w_engines_rarity = Vec::new();

    for weapon in &configs.weapon["LDABBJOAKGJ"] {
        let id = weapon.id;

        let Some(rarity) = configs.item["LDABBJOAKGJ"]
            .iter()
            .find(|i| i.id == id)
            .map(|i| i.rarity)
        else {
            continue;
        };

        w_engines_id.push(id);
        w_engines_rarity.push(rarity);
    }

    database::zzz::w_engines::set_all(&w_engines_id, &w_engines_rarity, pool).await?;

    Ok(())
}
