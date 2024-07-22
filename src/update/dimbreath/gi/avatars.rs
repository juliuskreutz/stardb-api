use regex::Regex;
use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut characters_id = Vec::new();
    let mut characters_rarity = Vec::new();

    for avatar in &configs.avatar_data {
        let id = avatar.id;

        let rarity = if avatar.quality == "QUALITY_PURPLE" {
            4
        } else {
            5
        };

        characters_id.push(id);
        characters_rarity.push(rarity);
    }

    database::gi::characters::set_all(&characters_id, &characters_rarity, pool).await?;

    Ok(())
}
