use std::collections::HashMap;

use actix_web::{get, web, HttpResponse, Responder};
use linked_hash_map::LinkedHashMap;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString};
use utoipa::{OpenApi, ToSchema};

mod id;

use crate::{
    database::{self, DbAchievement},
    Result,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements")),
    paths(get_achievements),
    components(
        schemas(
            Difficulty,
            Achievement
        )))
]
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
struct Series {
    name: String,
    achievements: Vec<Vec<Achievement>>,
}

#[derive(Serialize, ToSchema)]
struct Achievement {
    id: i64,
    series_name: String,
    title: String,
    description: String,
    jades: i32,
    hidden: bool,
    version: Option<String>,
    comment: Option<String>,
    reference: Option<String>,
    difficulty: Option<Difficulty>,
    percent: f64,
}

impl<T: AsRef<DbAchievement>> From<T> for Achievement {
    fn from(value: T) -> Self {
        let db_achievement = value.as_ref();

        Achievement {
            id: db_achievement.id,
            series_name: db_achievement.series_name.clone(),
            title: db_achievement.title.clone(),
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
            percent: db_achievement.percent.unwrap_or_default(),
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(id::configure).service(get_achievements);
}

#[utoipa::path(
    tag = "achievements",
    get,
    path = "/api/achievements",
    responses(
        (status = 200, description = "[Series]", body = Vec<Series>),
    )
)]
#[get("/api/achievements")]
async fn get_achievements(pool: web::Data<PgPool>) -> Result<impl Responder> {
    let db_achievements = database::get_achievements(&pool).await?;

    let mut series: LinkedHashMap<String, Vec<Vec<Achievement>>> = LinkedHashMap::new();
    let mut groupings: HashMap<i32, usize> = HashMap::new();

    for db_achievement in db_achievements {
        let achievements = series
            .entry(db_achievement.series_name.clone())
            .or_default();

        let grouping = db_achievement.grouping;
        let achievement = db_achievement.into();

        if let Some(grouping) = grouping {
            if let Some(&i) = groupings.get(&grouping) {
                achievements[i].push(achievement);
                continue;
            }

            groupings.insert(grouping, achievements.len());
        }

        achievements.push(vec![achievement]);
    }

    let series = series
        .into_iter()
        .map(|(name, achievements)| Series { name, achievements })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(series))
}
