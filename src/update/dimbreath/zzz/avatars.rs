use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut characters_id = Vec::new();
    let mut characters_rarity = Vec::new();

    for avatar in &configs.avatar["PEPPKLMFFBD"] {
        let id = avatar.id;

        let rarity = configs.item["PEPPKLMFFBD"]
            .iter()
            .find(|i| i.id == avatar.id)
            .map(|i| i.rarity)
            .unwrap_or_default();

        characters_id.push(id);
        characters_rarity.push(rarity);
    }

    database::zzz::characters::set_all(&characters_id, &characters_rarity, pool).await?;

    Ok(())
}
