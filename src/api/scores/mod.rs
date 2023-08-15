mod damage;
mod heal;
mod shield;

use actix_web::web;
mod achievements;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::{IntoParams, OpenApi, ToSchema};

use self::{
    achievements::ScoreAchievement, damage::ScoreDamage, heal::ScoreHeal, shield::ScoreShield,
};

#[derive(OpenApi)]
#[openapi(components(schemas(Region, ScoresAchievement, ScoresDamage, ScoresHeal, ScoresShield)))]
struct ApiDoc;

#[derive(Display, EnumString, Serialize, Deserialize, ToSchema, Clone, Copy)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Region {
    NA,
    EU,
    Asia,
    CN,
}

#[derive(Serialize, ToSchema)]
#[aliases(
    ScoresAchievement = Scores<ScoreAchievement>,
    ScoresDamage = Scores<ScoreDamage>,
    ScoresHeal = Scores<ScoreHeal>,
    ScoresShield = Scores<ScoreShield>
)]
pub struct Scores<T: Serialize> {
    pub count: i64,
    pub count_na: i64,
    pub count_eu: i64,
    pub count_asia: i64,
    pub count_cn: i64,
    pub count_query: i64,
    pub scores: Vec<T>,
}

#[derive(Deserialize, IntoParams)]
pub struct ScoresParams {
    pub region: Option<Region>,
    pub query: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievements::openapi());
    openapi.merge(damage::openapi());
    openapi.merge(heal::openapi());
    openapi.merge(shield::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievements::configure)
        .configure(damage::configure)
        .configure(heal::configure)
        .configure(shield::configure);
}
