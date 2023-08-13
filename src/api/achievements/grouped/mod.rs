use std::{collections::HashMap, time::Duration};

use actix_web::{get, rt, web, HttpResponse, Responder};
use futures::lock::Mutex;
use indexmap::IndexMap;
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::achievements::Achievement, database, Result};

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements/grouped")),
    paths(get_achievements_grouped),
    components(schemas(
        Groups,
        AchivementsGrouped
    ))
)]
struct ApiDoc;

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

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig, pool: PgPool) {
    let groups = web::Data::new(Mutex::new(Groups::default()));

    rt::spawn(async move {
        let minutes = 1;

        let mut timer = rt::time::interval(Duration::from_secs(60 * minutes));

        loop {
            timer.tick().await;

            let _ = update(&groups, &pool).await;
        }
    });

    cfg.service(get_achievements_grouped);
}

async fn update(groups: &web::Data<Mutex<Groups>>, pool: &PgPool) -> Result<()> {
    let db_achievements = database::get_achievements(pool).await?;

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

    let (achievement_count, jade_count) = series.iter().fold((0, 0), |(a_count, j_count), ag| {
        (a_count + ag.achievements.len(), j_count + ag.jade_count)
    });

    let user_count = database::get_distinct_username_count(pool).await?;

    *groups.lock().await = Groups {
        achievement_count,
        jade_count,
        user_count,
        series,
    };

    Ok(())
}

#[utoipa::path(
    tag = "achievements/grouped",
    get,
    path = "/api/achievements/grouped",
    responses(
        (status = 200, description = "Groups", body = Groups),
    )
)]
#[get("/api/achievements/grouped")]
async fn get_achievements_grouped(groups: web::Data<Mutex<Groups>>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(&*groups.lock().await))
}
