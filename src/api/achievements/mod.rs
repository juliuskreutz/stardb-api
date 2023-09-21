mod id;

use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString};
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{ApiResult, LanguageParams},
    database,
};

use super::Language;

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements")),
    paths(get_achievements),
    components(schemas(
        Difficulty,
        Language,
        Achievement
    ))
)]
struct ApiDoc;

#[derive(Display, EnumString, Serialize, Deserialize, ToSchema)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Serialize, ToSchema)]
struct Achievement {
    id: i64,
    series: i32,
    series_name: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<String>,
    gacha: bool,
    impossible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    set: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    related: Option<Vec<i64>>,
    percent: f64,
}

impl From<database::DbAchievement> for Achievement {
    fn from(db_achievement: database::DbAchievement) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            series_name: db_achievement.series_name.clone(),
            name: db_achievement.name.clone(),
            description: db_achievement.description.clone(),
            jades: db_achievement.jades,
            hidden: db_achievement.hidden,
            version: db_achievement.version.clone(),
            comment: db_achievement.comment.clone(),
            reference: db_achievement.reference.clone(),
            difficulty: db_achievement
                .difficulty
                .as_ref()
                .map(|d| d.parse().unwrap()),
            video: db_achievement.video.clone(),
            gacha: db_achievement.gacha,
            impossible: db_achievement.impossible,
            set: db_achievement.set,
            related: None,
            percent: db_achievement.percent,
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_achievements).configure(id::configure);
}

#[utoipa::path(
    tag = "achievements",
    get,
    path = "/api/achievements",
    params(LanguageParams),
    responses(
        (status = 200, description = "[Achievement]", body = Vec<Achievement>),
    )
)]
#[get("/api/achievements")]
async fn get_achievements(
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_achievements =
        database::get_achievements(&language_params.lang.to_string(), &pool).await?;

    let mut achievements = db_achievements
        .into_iter()
        .map(Achievement::from)
        .collect::<Vec<_>>();

    for achievement in &mut achievements {
        if let Some(set) = achievement.set {
            achievement.related = Some(database::get_related(achievement.id, set, &pool).await?);
        }
    }

    Ok(HttpResponse::Ok().json(achievements))
}
