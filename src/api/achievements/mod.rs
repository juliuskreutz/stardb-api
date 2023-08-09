mod grouped;
mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString};
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::{
    database::{self, DbAchievement},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements")),
    paths(get_achievements),
    components(schemas(
        Difficulty,
        Achievement
    ))
)]
struct ApiDoc;

#[derive(Display, EnumString, Serialize, Deserialize, ToSchema)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Serialize, ToSchema)]
struct Achievement {
    id: i64,
    series: i32,
    series_tag: String,
    series_name: String,
    tag: String,
    name: String,
    description: String,
    jades: i32,
    hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<Difficulty>,
    gacha: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    set: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    related: Option<Vec<i64>>,
    percent: f64,
}

#[derive(Deserialize, IntoParams)]
struct AchievementParams {
    series: Option<i32>,
    series_tag: Option<String>,
    hidden: Option<bool>,
    version: Option<String>,
    gacha: Option<bool>,
}

impl From<DbAchievement> for Achievement {
    fn from(db_achievement: DbAchievement) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            series_tag: db_achievement.series_tag,
            series_name: db_achievement.series_name,
            tag: db_achievement.tag,
            name: db_achievement.name,
            description: db_achievement.description,
            jades: db_achievement.jades,
            hidden: db_achievement.hidden,
            version: db_achievement.version,
            comment: db_achievement.comment,
            reference: db_achievement.reference,
            difficulty: db_achievement
                .difficulty
                .as_ref()
                .map(|d| d.parse().unwrap()),
            gacha: db_achievement.gacha,
            set: db_achievement.set,
            related: None,
            percent: db_achievement.percent.unwrap_or_default(),
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(grouped::openapi());
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_achievements)
        .configure(grouped::configure)
        .configure(id::configure);
}

#[utoipa::path(
    tag = "achievements",
    get,
    path = "/api/achievements",
    params(AchievementParams),
    responses(
        (status = 200, description = "[Achievement]", body = Vec<Achievement>),
    )
)]
#[get("/api/achievements")]
async fn get_achievements(
    achievement_params: web::Query<AchievementParams>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let db_achievements = database::get_achievements(
        achievement_params.series,
        achievement_params.series_tag.as_deref(),
        achievement_params.hidden,
        achievement_params.version.as_deref(),
        achievement_params.gacha,
        &pool,
    )
    .await?;

    let mut achievements = Vec::new();

    for db_achievement in db_achievements {
        let mut achievement = Achievement::from(db_achievement);

        if let Some(set) = achievement.set {
            achievement.related = Some(database::get_related(achievement.id, set, &pool).await?);
        };

        achievements.push(achievement);
    }

    Ok(HttpResponse::Ok().json(achievements))
}
