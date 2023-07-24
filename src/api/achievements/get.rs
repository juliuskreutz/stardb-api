use std::collections::HashMap;

use actix_web::{get, web, HttpResponse, Responder};
use linked_hash_map::LinkedHashMap;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::achievements::Achievement, database, Result};

#[derive(OpenApi)]
#[openapi(paths(get_achievements), components(schemas(Series)))]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_achievements);
}

#[derive(Serialize, ToSchema)]
struct Series {
    name: String,
    achievements: Vec<Vec<Achievement>>,
}

#[utoipa::path(
    get,
    path = "/api/achievements",
    responses(
        (status = 200, description = "[Series]", body = Vec<Series>),
    ),
    tag = "Achievements"
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
