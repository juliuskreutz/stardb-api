use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use actix_web::{
    get,
    rt::{self, time},
    web, HttpResponse, Responder,
};
use anyhow::Result;
use futures::lock::Mutex;
use indexmap::IndexMap;
use serde::Serialize;
use sqlx::PgPool;
use strum::{EnumString, IntoEnumIterator};

use crate::api::{ApiResult, LanguageParams};

use crate::api::Language;

#[derive(Default)]
pub struct AchievementTrackerCache {
    achievement_tracker_map: Mutex<HashMap<Language, AchievementTracker>>,
}

#[derive(Default, Serialize)]
struct AchievementTracker {
    achievement_count: usize,
    jade_count: i32,
    user_count: i64,
    languages: Vec<serde_json::Value>,
    language: Language,
    versions: Vec<String>,
    series: Vec<Series>,
}

#[derive(Serialize)]
struct Series {
    series: String,
    achievement_count: usize,
    jade_count: i32,
    achievements: Vec<Vec<Achievement>>,
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
    gacha: bool,
    percent: f64,
}

#[derive(EnumString, Serialize)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl From<stardb_database::DbAchievement> for Achievement {
    fn from(db_achievement: stardb_database::DbAchievement) -> Self {
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
            gacha: db_achievement.gacha,
            percent: db_achievement.percent.unwrap_or_default(),
        }
    }
}

pub fn cache(pool: PgPool) -> web::Data<AchievementTrackerCache> {
    let achievement_tracker_cache = web::Data::new(AchievementTrackerCache::default());

    {
        let achievement_tracker_cache = achievement_tracker_cache.clone();

        rt::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let start = Instant::now();

                if let Err(e) = update(&achievement_tracker_cache, &pool).await {
                    log::error!(
                        "Achievement Tracker update failed with {e} in {}s",
                        start.elapsed().as_secs_f64()
                    );
                } else {
                    log::info!(
                        "Achievement Tracker update succeeded in {}s",
                        start.elapsed().as_secs_f64()
                    );
                }
            }
        });
    }

    achievement_tracker_cache
}

async fn update(
    achievement_tracker_cache: &web::Data<AchievementTrackerCache>,
    pool: &PgPool,
) -> Result<()> {
    let mut achievement_tracker_map = HashMap::new();

    for language in Language::iter() {
        let db_achievements =
            stardb_database::get_achievements(&language.to_string(), pool).await?;

        let mut series: IndexMap<String, Vec<Vec<Achievement>>> = IndexMap::new();
        let mut groupings: HashMap<i32, usize> = HashMap::new();
        let mut versions: HashSet<String> = HashSet::new();

        for db_achievement in db_achievements {
            let achievements = series
                .entry(db_achievement.series_name.clone())
                .or_default();

            if let Some(version) = &db_achievement.version {
                versions.insert(version.clone());
            }

            if let Some(set) = db_achievement.set {
                if let Some(&i) = groupings.get(&set) {
                    achievements[i].push(Achievement::from(db_achievement));
                    continue;
                }

                groupings.insert(set, achievements.len());
            }

            achievements.push(vec![Achievement::from(db_achievement)]);
        }

        let series = series
            .into_iter()
            .map(|(series, achievements)| Series {
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

        let user_count = stardb_database::get_distinct_username_count(pool).await?;
        let languages = Language::iter()
            .map(|l| {
                serde_json::json!({
                    l.to_string(): l.name()
                })
            })
            .collect::<Vec<_>>();
        let mut versions: Vec<_> = versions.into_iter().collect();
        versions.sort();

        let achievement_tracker = AchievementTracker {
            achievement_count,
            jade_count,
            user_count,
            languages,
            language,
            versions,
            series,
        };

        achievement_tracker_map.insert(language, achievement_tracker);
    }

    *achievement_tracker_cache
        .achievement_tracker_map
        .lock()
        .await = achievement_tracker_map;

    Ok(())
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_achievemenent_tracker);
}

#[get("/api/private/achievement-tracker")]
async fn get_achievemenent_tracker(
    language_params: web::Query<LanguageParams>,
    achievement_tracker_cache: web::Data<AchievementTrackerCache>,
) -> ApiResult<impl Responder> {
    Ok(HttpResponse::Ok().json(
        &achievement_tracker_cache
            .achievement_tracker_map
            .lock()
            .await[&language_params.lang],
    ))
}
