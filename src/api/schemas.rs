use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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
