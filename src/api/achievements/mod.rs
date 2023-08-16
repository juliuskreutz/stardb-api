mod id;

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use actix_web::{get, rt, web, HttpResponse, Responder};
use futures::lock::Mutex;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString, IntoEnumIterator};
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::{
    api::{ApiResult, LanguageParams},
    database::{self, DbAchievement},
};

use super::Language;

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements")),
    paths(get_achievements),
    components(schemas(
        Difficulty,
        Language,
        Layout,
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

#[derive(Default, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
enum Layout {
    #[default]
    Flat,
    Grouped,
}

#[derive(Deserialize, IntoParams)]
struct AchievementParams {
    #[serde(default)]
    layout: Layout,
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
    gacha: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    set: Option<i32>,
    percent: f64,
}

#[derive(Default, Serialize, ToSchema)]
struct Groups {
    achievement_count: usize,
    jade_count: i32,
    user_count: i64,
    series: Vec<AchivementsGrouped>,
}

#[derive(Serialize, ToSchema)]
struct AchivementsGrouped {
    series: String,
    achievement_count: usize,
    jade_count: i32,
    achievements: Vec<Vec<Achievement>>,
}

impl From<DbAchievement> for Achievement {
    fn from(db_achievement: DbAchievement) -> Self {
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
            gacha: db_achievement.gacha,
            set: db_achievement.set,
            percent: db_achievement.percent.unwrap_or_default(),
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig, pool: PgPool) {
    let achievements = web::Data::new(Mutex::new(HashMap::new()));
    let groups = web::Data::new(Mutex::new(HashMap::new()));

    {
        let achievements = achievements.clone();
        let groups = groups.clone();
        let pool = pool.clone();

        rt::spawn(async move {
            let minutes = 1;

            let mut timer = rt::time::interval(Duration::from_secs(60 * minutes));

            loop {
                timer.tick().await;

                let start = Instant::now();

                if let Err(e) = update(&achievements, &groups, &pool).await {
                    log::error!(
                        "Achievements update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    log::info!(
                        "Achievements update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });
    }

    cfg.app_data(achievements)
        .app_data(groups)
        .service(get_achievements)
        .configure(id::configure);
}

async fn update(
    achievements: &web::Data<Mutex<HashMap<Language, Vec<Achievement>>>>,
    groups: &web::Data<Mutex<HashMap<Language, Groups>>>,
    pool: &PgPool,
) -> anyhow::Result<()> {
    let mut achievements_map = HashMap::new();
    let mut groups_map = HashMap::new();

    for language in Language::iter() {
        let db_achievements = database::get_achievements(&language.to_string(), pool).await?;

        achievements_map.insert(
            language,
            db_achievements
                .clone()
                .into_iter()
                .map(Achievement::from)
                .collect(),
        );

        let mut series: IndexMap<String, Vec<Vec<Achievement>>> = IndexMap::new();
        let mut groupings: HashMap<i32, usize> = HashMap::new();

        for db_achievement in db_achievements {
            let achievements = series
                .entry(db_achievement.series_name.clone())
                .or_default();

            let achievement = Achievement::from(db_achievement);

            if let Some(set) = achievement.set {
                if let Some(&i) = groupings.get(&set) {
                    achievements[i].push(achievement);
                    continue;
                }

                groupings.insert(set, achievements.len());
            }

            achievements.push(vec![achievement]);
        }

        let series = series
            .into_iter()
            .map(|(series, achievements)| AchivementsGrouped {
                series,
                achievement_count: achievements.len(),
                jade_count: achievements.iter().map(|a| a[0].jades).sum(),
                achievements,
            })
            .collect::<Vec<_>>();

        let (achievement_count, jade_count) =
            series.iter().fold((0, 0), |(a_count, j_count), ag| {
                (a_count + ag.achievements.len(), j_count + ag.jade_count)
            });

        let user_count = database::get_distinct_username_count(pool).await?;

        let group = Groups {
            achievement_count,
            jade_count,
            user_count,
            series,
        };

        groups_map.insert(language, group);
    }

    *achievements.lock().await = achievements_map;
    *groups.lock().await = groups_map;

    Ok(())
}

#[utoipa::path(
    tag = "achievements",
    get,
    path = "/api/achievements",
    params(LanguageParams, AchievementParams),
    responses(
        (status = 200, description = "[Achievement]", body = Vec<Achievement>),
    )
)]
#[get("/api/achievements")]
async fn get_achievements(
    language_params: web::Query<LanguageParams>,
    achievement_params: web::Query<AchievementParams>,
    achievements: web::Data<Mutex<HashMap<Language, Vec<Achievement>>>>,
    groups: web::Data<Mutex<HashMap<Language, Groups>>>,
) -> ApiResult<impl Responder> {
    Ok(match achievement_params.layout {
        Layout::Flat => HttpResponse::Ok().json(&achievements.lock().await[&language_params.lang]),
        Layout::Grouped => HttpResponse::Ok().json(&groups.lock().await[&language_params.lang]),
    })
}
