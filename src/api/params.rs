use serde::Deserialize;
use utoipa::IntoParams;

use super::schemas::{CharacterDamage, Region};

#[derive(Deserialize, IntoParams)]
pub struct ScoresParams {
    pub region: Option<Region>,
    pub query: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, IntoParams)]
pub struct DamageParams {
    pub character: Option<CharacterDamage>,
    pub support: Option<bool>,
}

#[derive(Deserialize, IntoParams)]
pub struct SubmissionsParams {
    pub uid: Option<i64>,
}
