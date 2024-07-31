use sqlx::PgPool;

use crate::database;

use super::Configs;

pub async fn update(configs: &Configs, pool: &PgPool) -> anyhow::Result<()> {
    let mut light_cones_id = Vec::new();
    let mut light_cones_rarity = Vec::new();

    for item_config_equipment in &configs.equipment_config {
        let id = item_config_equipment.id;

        let rarity = match item_config_equipment.rarity.as_str() {
            "CombatPowerLightconeRarity3" => 3,
            "CombatPowerLightconeRarity4" => 4,
            "CombatPowerLightconeRarity5" => 5,
            _ => unreachable!(),
        };

        light_cones_id.push(id);
        light_cones_rarity.push(rarity);
    }

    database::set_all_light_cones(&light_cones_id, &light_cones_rarity, pool).await?;

    Ok(())
}
