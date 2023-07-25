use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use utoipa::ToSchema;
use uuid::Uuid;

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
    pub scores: Vec<T>,
}

#[derive(Display, EnumString, Serialize, Deserialize, ToSchema)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Region {
    NA,
    EU,
    Asia,
    CN,
}

#[derive(Serialize, ToSchema)]
pub struct ScoreAchievement {
    pub global_rank: i64,
    pub regional_rank: i64,
    pub uid: i64,
    pub region: Region,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub achievement_count: i32,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct ScoreDamage {
    pub global_rank: i64,
    pub regional_rank: i64,
    pub uid: i64,
    pub character: String,
    pub support: bool,
    pub damage: i32,
    pub video: String,
    pub region: Region,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct ScoreHeal {
    pub global_rank: i64,
    pub regional_rank: i64,
    pub uid: i64,
    pub heal: i32,
    pub video: String,
    pub region: Region,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct ScoreShield {
    pub global_rank: i64,
    pub regional_rank: i64,
    pub uid: i64,
    pub shield: i32,
    pub video: String,
    pub region: Region,
    pub name: String,
    pub level: i32,
    pub signature: String,
    pub avatar_icon: String,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct SubmissionDamage {
    #[schema(value_type = String)]
    pub uuid: Uuid,
    pub uid: i64,
    pub character: String,
    pub support: bool,
    pub damage: i32,
    pub video: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct SubmissionHeal {
    #[schema(value_type = String)]
    pub uuid: Uuid,
    pub uid: i64,
    pub heal: i32,
    pub video: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, ToSchema)]
pub struct SubmissionShield {
    #[schema(value_type = String)]
    pub uuid: Uuid,
    pub uid: i64,
    pub shield: i32,
    pub video: String,
    pub created_at: NaiveDateTime,
}

#[derive(Deserialize, ToSchema)]
pub struct SubmissionDamageUpdate {
    pub uid: i64,
    pub character: String,
    pub support: bool,
    pub damage: i32,
    pub video: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SubmissionHealUpdate {
    pub uid: i64,
    pub heal: i32,
    pub video: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SubmissionShieldUpdate {
    pub uid: i64,
    pub shield: i32,
    pub video: String,
}

#[derive(Deserialize, ToSchema)]
pub struct DamageUpdate {
    pub character: String,
    pub support: bool,
    pub damage: i32,
    pub video: String,
}

#[derive(Deserialize, ToSchema)]
pub struct HealUpdate {
    pub heal: i32,
    pub video: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ShieldUpdate {
    pub shield: i32,
    pub video: String,
}
