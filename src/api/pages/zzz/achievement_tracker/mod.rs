use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    sync::Mutex,
    time::{Duration, Instant},
};

use actix_session::Session;
use actix_web::{
    get,
    rt::{self, Runtime},
    web, HttpResponse, Responder,
};
use async_rwlock::RwLock;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::IntoEnumIterator;
use utoipa::OpenApi;

use crate::{
    api::{private, ApiResult, Language, LanguageParams},
    database, Difficulty,
};

lazy_static::lazy_static! {
    static ref CACHE: Mutex<Option<web::Data<ZzzAchievementTrackerCache>>> = Mutex::new(None);
}

#[derive(OpenApi)]
#[openapi(paths(get_zzz_achievement_tracker))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig, pool: PgPool) {
    let data = CACHE
        .lock()
        .unwrap()
        .get_or_insert_with(|| cache(pool))
        .clone();

    cfg.service(get_zzz_achievement_tracker).app_data(data);
}

#[derive(Default)]
pub struct ZzzAchievementTrackerCache {
    achievement_tracker_map: RwLock<HashMap<Language, AchievementTracker>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AchievementTracker {
    achievement_count: usize,
    achievement_count_current: usize,
    currency_count: i32,
    currency_count_current: i32,
    user_count: i64,
    language: Language,
    versions: Vec<String>,
    series: Vec<Series>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Series {
    series: String,
    achievement_count: usize,
    achievement_count_current: usize,
    currency_count: i32,
    currency_count_current: i32,
    achievement_groups: Vec<AchievementGroup>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AchievementGroup {
    complete: Option<i32>,
    favorite: Option<i32>,
    achievements: Vec<Achievement>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Achievement {
    id: i32,
    series: i32,
    series_name: String,
    series_index: usize,
    name: String,
    description: String,
    currency: i32,
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

impl From<database::zzz::achievements::DbAchievement> for Achievement {
    fn from(db_achievement: database::zzz::achievements::DbAchievement) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            series_name: db_achievement.series_name.clone(),
            series_index: 0,
            name: db_achievement.name.clone(),
            description: db_achievement.description.clone(),
            currency: db_achievement.polychromes,
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

pub fn cache(pool: PgPool) -> web::Data<ZzzAchievementTrackerCache> {
    let achievement_tracker_map = RwLock::new(
        if let Ok(file) = File::open("cache/zzz_achievement_tracker_map.json") {
            if let Ok(achievement_tracker_map) = serde_json::from_reader::<
                _,
                HashMap<Language, AchievementTracker>,
            >(BufReader::new(file))
            {
                achievement_tracker_map
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        },
    );

    let achievement_tracker_cache = web::Data::new(ZzzAchievementTrackerCache {
        achievement_tracker_map,
    });

    {
        let achievement_tracker_cache = achievement_tracker_cache.clone();

        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();

            let handle = rt.spawn(async move {
                let mut success = true;

                let mut interval = rt::time::interval(Duration::from_secs(60));

                loop {
                    if success {
                        interval.tick().await;
                    }

                    let start = Instant::now();

                    if let Err(e) =
                        update_achievement_tracker(achievement_tracker_cache.clone(), pool.clone())
                            .await
                    {
                        success = false;

                        error!(
                            "Zzz Achievement Tracker update failed with {e} in {}s",
                            start.elapsed().as_secs_f64()
                        );
                    } else {
                        success = true;

                        info!(
                            "Zzz Achievement Tracker update succeeded in {}s",
                            start.elapsed().as_secs_f64()
                        );
                    }
                }
            });

            rt.block_on(handle).unwrap();
        });
    }

    achievement_tracker_cache
}

async fn update_achievement_tracker(
    achievement_tracker_cache: web::Data<ZzzAchievementTrackerCache>,
    pool: PgPool,
) -> anyhow::Result<()> {
    let mut achievement_tracker_map = HashMap::new();

    let user_count = database::zzz::users_achievements_completed::get_user_count(&pool).await?;

    for language in Language::iter() {
        let achievements = database::zzz::achievements::get_all(language, &pool).await?;

        info!("{}", achievements.len());

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
                    achievement_count_current: 0,
                    currency_count: 0,
                    currency_count_current: 0,
                    achievement_groups: Vec::new(),
                });
            }

            if achievement
                .set
                .map(|set| current_set == Some(set))
                .unwrap_or(false)
            {
                let mut achievement: Achievement = achievement.into();
                achievement.series_index = series.len() - 1;

                series
                    .last_mut()
                    .unwrap()
                    .achievement_groups
                    .last_mut()
                    .unwrap()
                    .achievements
                    .push(achievement);
            } else {
                current_set = achievement.set;

                let mut achievement: Achievement = achievement.into();
                achievement.series_index = series.len() - 1;

                series
                    .last_mut()
                    .unwrap()
                    .achievement_groups
                    .push(AchievementGroup {
                        complete: None,
                        favorite: None,
                        achievements: vec![achievement],
                    });
            }
        }

        let mut achievement_count = 0;
        let mut currency_count = 0;

        for series in series.iter_mut() {
            series.achievement_count = series.achievement_groups.len();
            series.currency_count = series
                .achievement_groups
                .iter()
                .map(|group| group.achievements[0].currency)
                .sum();

            achievement_count += series.achievement_count;
            currency_count += series.currency_count;
        }

        let mut versions = versions.into_iter().collect::<Vec<_>>();
        versions.sort_unstable();

        let achievement_tracker = AchievementTracker {
            achievement_count,
            achievement_count_current: 0,
            currency_count,
            currency_count_current: 0,
            user_count,
            language,
            versions,
            series,
        };

        achievement_tracker_map.insert(language, achievement_tracker);
    }

    std::fs::write(
        "cache/zzz_achievement_tracker_map.json",
        serde_json::to_vec(&achievement_tracker_map)?,
    )?;

    *achievement_tracker_cache
        .achievement_tracker_map
        .write()
        .await = achievement_tracker_map;

    Ok(())
}

#[utoipa::path(
    tag = "pages",
    get,
    path = "/api/pages/zzz/achievement-tracker",
    params(LanguageParams),
    security(("api_key" = [])),
    responses(
        (status = 200, description = "AchievementTracker"),
    )
)]
#[get("/api/pages/zzz/achievement-tracker", guard = "private")]
async fn get_zzz_achievement_tracker(
    session: Session,
    language_params: web::Query<LanguageParams>,
    achievement_tracker_cache: web::Data<ZzzAchievementTrackerCache>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let mut achievement_tracker = achievement_tracker_cache
        .achievement_tracker_map
        .read()
        .await[&language_params.lang]
        .clone();

    if let Ok(Some(username)) = session.get::<String>("username") {
        let completed =
            database::zzz::users_achievements_completed::get_by_username(&username, &pool)
                .await?
                .into_iter()
                .map(|c| c.id)
                .collect::<HashSet<_>>();
        let favorites =
            database::zzz::users_achievements_favorites::get_by_username(&username, &pool)
                .await?
                .into_iter()
                .map(|c| c.id)
                .collect::<HashSet<_>>();

        let mut achievement_count_current_total = 0;
        let mut currency_count_current_total = 0;

        for series in achievement_tracker.series.iter_mut() {
            let mut achievement_count_current = 0;
            let mut currency_count_current = 0;

            for group in series.achievement_groups.iter_mut() {
                let complete = group
                    .achievements
                    .iter()
                    .map(|c| c.id)
                    .find(|id| completed.contains(id));

                let favorite = group
                    .achievements
                    .iter()
                    .map(|c| c.id)
                    .find(|id| favorites.contains(id));

                group.complete = complete;
                group.favorite = favorite;

                if let Some(complete) = complete {
                    achievement_count_current += 1;

                    currency_count_current += group
                        .achievements
                        .iter()
                        .find(|a| a.id == complete)
                        .unwrap()
                        .currency;
                }
            }

            series.achievement_count_current = achievement_count_current;
            series.currency_count_current = currency_count_current;

            achievement_count_current_total += achievement_count_current;
            currency_count_current_total += currency_count_current;
        }

        achievement_tracker.achievement_count_current = achievement_count_current_total;
        achievement_tracker.currency_count_current = currency_count_current_total;
    }

    Ok(HttpResponse::Ok().json(achievement_tracker))
}
