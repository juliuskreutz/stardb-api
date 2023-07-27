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
    paths(get_achievements, get_achievements_grouped),
    components(schemas(
        Difficulty,
        Achievement,
        Group
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
    series: String,
    title: String,
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
    set: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    related: Option<Vec<i64>>,
    percent: f64,
}

#[derive(Serialize, ToSchema)]
struct Group {
    series: String,
    achievements: Vec<Vec<Achievement>>,
}

impl From<DbAchievement> for Achievement {
    fn from(db_achievement: DbAchievement) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            title: db_achievement.title,
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
            set: db_achievement.set,
            related: None,
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
    cfg.service(get_achievements)
        .service(get_achievements_grouped)
        .configure(id::configure);
}

#[utoipa::path(
    tag = "achievements",
    get,
    path = "/api/achievements",
    responses(
        (status = 200, description = "[Achievement]", body = Vec<Achievement>),
    )
)]
#[get("/api/achievements")]
async fn get_achievements(pool: web::Data<PgPool>) -> Result<impl Responder> {
    let db_achievements = database::get_achievements(&pool).await?;

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

#[utoipa::path(
    tag = "achievements",
    get,
    path = "/api/achievements/grouped",
    responses(
        (status = 200, description = "[Group]", body = Vec<Group>),
    )
)]
#[get("/api/achievements/grouped")]
async fn get_achievements_grouped(pool: web::Data<PgPool>) -> Result<impl Responder> {
    let db_achievements = database::get_achievements(&pool).await?;

    let mut series: LinkedHashMap<String, Vec<Vec<Achievement>>> = LinkedHashMap::new();
    let mut groupings: HashMap<i32, usize> = HashMap::new();

    for db_achievement in db_achievements {
        let achievements = series.entry(db_achievement.series.clone()).or_default();

        let mut achievement = Achievement::from(db_achievement);

        if let Some(set) = achievement.set {
            achievement.related = Some(database::get_related(achievement.id, set, &pool).await?);

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
        .map(|(series, achievements)| Group {
            series,
            achievements,
        })
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(series))
}
