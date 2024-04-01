use std::collections::HashSet;

use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use strum::EnumString;
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, Language, LanguageParams},
    database,
};

#[derive(OpenApi)]
#[openapi(paths(get_achievement_tracker))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_achievement_tracker);
}

#[derive(Default, Serialize)]
struct AchievementTracker {
    achievement_count: usize,
    jade_count: i32,
    user_count: i64,
    language: Language,
    versions: Vec<String>,
    series: Vec<Series>,
}

#[derive(Serialize)]
struct Series {
    series: String,
    achievement_count: usize,
    jade_count: i32,
    achievement_groups: Vec<AchievementGroup>,
}

#[derive(Serialize)]
struct AchievementGroup {
    completed: Option<i64>,
    favorite: Option<i64>,
    achievements: Vec<Achievement>,
}

#[derive(Serialize)]
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
    difficulty: Option<Difficulty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<String>,
    gacha: bool,
    impossible: bool,
    percent: f64,
}

#[derive(EnumString, Serialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl From<database::DbAchievementTracker> for Achievement {
    fn from(db_achievement: database::DbAchievementTracker) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            series_name: db_achievement.series_name.clone(),
            name: db_achievement.name.clone(),
            description: db_achievement.description.clone(),
            jades: db_achievement.jades,
            hidden: db_achievement.hidden,
            version: db_achievement.version.clone(),
            difficulty: db_achievement
                .difficulty
                .as_ref()
                .map(|d| d.parse().unwrap()),
            video: db_achievement.video.clone(),
            gacha: db_achievement.gacha,
            impossible: db_achievement.impossible,
            percent: db_achievement.percent,
        }
    }
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/achievement-tracker",
    params(LanguageParams),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "AchievementTracker"),
    )
)]
#[get("/api/pages/achievement-tracker", guard = "private")]
async fn get_achievement_tracker(
    session: Session,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let language = language_params.lang;

    let username = if let Ok(Some(username)) = session.get::<String>("username") {
        Some(username)
    } else {
        None
    };

    let achievements =
        database::get_achievement_tracker(&username.unwrap(), &language.to_string(), &pool).await?;

    let achievement_count = achievements.len();
    let jade_count = achievements.iter().map(|a| a.jades).sum();

    let mut versions = HashSet::new();
    let mut series = Vec::new();

    let mut current_series = None;
    let mut current_set = None;

    for achievement in achievements
        .into_iter()
        .filter(|a| !(a.hidden && a.impossible))
    {
        versions.insert(achievement.version.clone().unwrap_or_default());

        if current_series != Some(achievement.series_name.clone()) {
            current_series = Some(achievement.series_name.clone());

            series.push(Series {
                series: achievement.series_name.clone(),
                achievement_count: 0,
                jade_count: 0,
                achievement_groups: Vec::new(),
            });
        }

        if achievement
            .set
            .map(|set| current_set == Some(set))
            .unwrap_or(false)
        {
            let group = series
                .last_mut()
                .unwrap()
                .achievement_groups
                .last_mut()
                .unwrap();

            if Some(true) == achievement.completed {
                group.completed = Some(achievement.id);
            }

            if Some(true) == achievement.favorite {
                group.favorite = Some(achievement.id);
            }

            group.achievements.push(achievement.into());
        } else {
            current_set = achievement.set;

            series
                .last_mut()
                .unwrap()
                .achievement_groups
                .push(AchievementGroup {
                    completed: None,
                    favorite: None,
                    achievements: vec![achievement.into()],
                });
        }
    }

    for series in series.iter_mut() {
        series.achievement_count = series.achievement_groups.len();
        series.jade_count = series
            .achievement_groups
            .iter()
            .map(|group| group.achievements[0].jades)
            .sum();
    }

    let achievement_tracker = AchievementTracker {
        achievement_count,
        jade_count,
        user_count: database::get_users_achievements_completed_user_count(&pool).await?,
        language,
        versions: versions.into_iter().collect(),
        series,
    };

    Ok(HttpResponse::Ok().json(achievement_tracker))
}
