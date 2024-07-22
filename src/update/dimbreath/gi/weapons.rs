use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut weapons_id = Vec::new();
    let mut weapons_rarity = Vec::new();

    for weapon in &configs.weapon_data {
        let id = weapon.id;

        let rarity = weapon.rank;

        weapons_id.push(id);
        weapons_rarity.push(rarity);
    }

    database::gi::weapons::set_all(&weapons_id, &weapons_rarity, pool).await?;

    Ok(())
}
