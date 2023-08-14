mod grouped;
mod id;

use std::{collections::HashMap, time::Duration};

use actix_web::{get, rt, web, HttpResponse, Responder};
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString, IntoEnumIterator};
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::LanguageParams,
    database::{self, DbAchievement},
    Result,
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
    openapi.merge(grouped::openapi());
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig, pool: PgPool) {
    let achievements = web::Data::new(Mutex::new(HashMap::new()));

    {
        let achievements = achievements.clone();
        let pool = pool.clone();

        rt::spawn(async move {
            let minutes = 1;

            let mut timer = rt::time::interval(Duration::from_secs(60 * minutes));

            loop {
                timer.tick().await;

                let _ = update(&achievements, &pool).await;
            }
        });
    }

    cfg.app_data(achievements)
        .service(get_achievements)
        .configure(|cfg| grouped::configure(cfg, pool))
        .configure(id::configure);
}

async fn update(
    achievements: &web::Data<Mutex<HashMap<Language, Vec<Achievement>>>>,
    pool: &PgPool,
) -> Result<()> {
    let mut map = HashMap::new();

    for language in Language::iter() {
        let achievements = database::get_achievements(&language.to_string(), pool)
            .await?
            .clone()
            .into_iter()
            .map(Achievement::from)
            .collect();

        map.insert(language, achievements);
    }

    *achievements.lock().await = map;

    Ok(())
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
    achievements: web::Data<Mutex<HashMap<Language, Vec<Achievement>>>>,
) -> Result<impl Responder> {
    let achievements = &achievements.lock().await[&language_params.lang];

    Ok(HttpResponse::Ok().json(achievements))
}
