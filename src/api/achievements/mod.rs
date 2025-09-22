mod id;

use actix_session::Session;
use actix_web::{get, put, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{ApiResult, LanguageParams},
    database, Difficulty,
};

use crate::Language;

#[derive(OpenApi)]
#[openapi(
    tags((name = "achievements")),
    paths(get_achievements, put_achievements),
    components(schemas(
        Language,
        Achievement,
        UpdateAchievement,
    ))
)]
struct ApiDoc;

#[derive(Serialize, ToSchema)]
struct Achievement {
    id: i32,
    series: i32,
    series_name: String,
    name: String,
    description: String,
    currency: i32,
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
    video: Option<String>,
    gacha: bool,
    timegated: Option<String>,
    missable: bool,
    impossible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    set: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    related: Option<Vec<i32>>,
    percent: f64,
}

impl From<database::achievements::DbAchievement> for Achievement {
    fn from(db_achievement: database::achievements::DbAchievement) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            series_name: db_achievement.series_name.clone(),
            name: db_achievement.name.clone(),
            description: db_achievement.description.clone(),
            currency: db_achievement.jades,
            hidden: db_achievement.hidden,
            version: db_achievement.version.clone(),
            comment: db_achievement.comment.clone(),
            reference: db_achievement.reference.clone(),
            difficulty: db_achievement
                .difficulty
                .as_ref()
                .map(|d| d.parse().unwrap()),
            video: db_achievement.video.clone(),
            gacha: db_achievement.gacha,
            timegated: db_achievement.timegated,
            missable: db_achievement.missable,
            impossible: db_achievement.impossible,
            set: db_achievement.set,
            related: None,
            percent: if !db_achievement.impossible {
                db_achievement.percent.unwrap_or_default()
            } else {
                0.0
            },
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
        .service(put_achievements)
        .configure(id::configure);
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
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let db_achievements = database::achievements::get_all(language_params.lang, &pool).await?;

    let mut achievements = db_achievements
        .into_iter()
        .map(Achievement::from)
        .collect::<Vec<_>>();

    for achievement in &mut achievements {
        if let Some(set) = achievement.set {
            achievement.related = Some(
                database::achievements::get_all_related_ids(achievement.id, set, &pool).await?,
            );
        }
    }

    Ok(HttpResponse::Ok().json(achievements))
}

#[derive(serde::Deserialize, ToSchema)]
struct UpdateAchievement {
    id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    difficulty: Option<Difficulty>,
    #[serde(skip_serializing_if = "Option::is_none")]
    video: Option<String>,
    gacha: Option<bool>,
    timegated: Option<String>,
    missable: Option<bool>,
    impossible: Option<bool>,
    set: Option<i32>,
}

#[utoipa::path(
    tag = "achievements",
    put,
    path = "/api/achievements",
    responses(
        (status = 200, description = "Updated Achievement", body = Vec<UpdateAchievement>),
    )
)]
#[put("/api/achievements")]
async fn put_achievements(
    session: Session,
    achievements: web::Json<Vec<UpdateAchievement>>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let admin = if let Ok(Some(username)) = session.get::<String>("username") {
        database::admins::exists(&username, &pool).await?
    } else {
        false
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    for achievement in achievements.iter() {
        let update_achievement = database::achievements::DbUpdateAchievement {
            id: achievement.id,
            version: achievement.version.clone(),
            comment: achievement.comment.clone(),
            reference: achievement.reference.clone(),
            difficulty: achievement.difficulty.map(|d| d.to_string()),
            video: achievement.video.clone(),
            gacha: achievement.gacha,
            timegated: achievement.timegated.clone(),
            missable: achievement.missable,
            impossible: achievement.impossible,
            set: achievement.set,
        };

        database::achievements::update_achievement_by_id(&update_achievement, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
