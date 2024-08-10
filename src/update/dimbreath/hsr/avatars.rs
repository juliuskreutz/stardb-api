use regex::Regex;
use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let rarity_re = Regex::new(r"CombatPowerAvatarRarityType(\d+)")?;

    let mut characters_id = Vec::new();
    let mut characters_rarity = Vec::new();

    for avatar_config in &configs.avatar_config {
        let id = avatar_config.id;

        let rarity: i32 = rarity_re
            .captures(&avatar_config.rarity)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or_default();

        characters_id.push(id);
        characters_rarity.push(rarity);
    }

    database::characters::set_all(&characters_id, &characters_rarity, pool).await?;

    Ok(())
}
