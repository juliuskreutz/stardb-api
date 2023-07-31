use std::collections::HashMap;

use actix_web::{get, web, HttpResponse, Responder};
use linked_hash_map::LinkedHashMap;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::achievements::Achievement, database, Result};

#[derive(OpenApi)]
#[openapi(tags((name = "achievements/grouped")), paths(get_achievements_grouped), components(schemas(Group)))]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct Group {
    series: String,
    achievements: Vec<Vec<Achievement>>,
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_achievements_grouped);
}

#[utoipa::path(
    tag = "achievements/grouped",
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
        let achievements = series
            .entry(db_achievement.series_name.clone())
            .or_default();

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
