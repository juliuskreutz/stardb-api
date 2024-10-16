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

#[derive(OpenApi)]
#[openapi(
    tags((name = "gi/achievements")),
    paths(get_gi_achievements, put_gi_achievements),
    components(schemas(Achievement, UpdateAchievement))
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
    #[serde(skip_serializing_if = "Option::is_none")]
    timegated: Option<String>,
    missable: bool,
    impossible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    set: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    related: Option<Vec<i32>>,
    percent: f64,
}

impl From<database::gi::achievements::DbAchievement> for Achievement {
    fn from(db_achievement: database::gi::achievements::DbAchievement) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            series_name: db_achievement.series_name.clone(),
            name: db_achievement.name.clone(),
            description: db_achievement.description.clone(),
            currency: db_achievement.primogems,
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
            timegated: db_achievement.timegated.clone(),
            missable: db_achievement.missable,
            impossible: db_achievement.impossible,
            set: db_achievement.set,
            related: None,
            percent: (!db_achievement.impossible)
                .then_some(db_achievement.percent.unwrap_or_default())
                .unwrap_or_default(),
        }
    }
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_gi_achievements)
        .service(put_gi_achievements)
        .configure(id::configure);
}

#[utoipa::path(
    tag = "gi/achievements",
    get,
    path = "/api/gi/achievements",
    params(LanguageParams),
    responses(
        (status = 200, description = "[Achievement]", body = Vec<Achievement>),
    )
)]
#[get("/api/gi/achievements")]
async fn get_gi_achievements(
    session: Session,
    language_params: web::Query<LanguageParams>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let admin = if let Ok(Some(username)) = session.get::<String>("username") {
        database::admins::exists(&username, &pool).await?
    } else {
        false
    };

    let mut db_achievements =
        database::gi::achievements::get_all(language_params.lang, &pool).await?;

    if !admin {
        db_achievements.retain(|a| !(a.hidden && a.impossible));
    }

    let mut achievements = db_achievements
        .into_iter()
        .map(Achievement::from)
        .collect::<Vec<_>>();

    for achievement in &mut achievements {
        if let Some(set) = achievement.set {
            achievement.related = Some(
                database::gi::achievements::get_all_related_ids(achievement.id, set, &pool).await?,
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
}

#[utoipa::path(
    tag = "gi/achievements",
    put,
    path = "/api/gi/achievements",
    responses(
        (status = 200, description = "Updated Achievement", body = Vec<UpdateAchievement>),
    )
)]
#[put("/api/gi/achievements")]
async fn put_gi_achievements(
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
        let update_achievement = database::gi::achievements::DbUpdateAchievement {
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
        };

        database::gi::achievements::update_achievement_by_id(&update_achievement, &pool).await?;
    }

    Ok(HttpResponse::Ok().finish())
}
