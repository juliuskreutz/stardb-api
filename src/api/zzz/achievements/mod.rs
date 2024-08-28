mod id;

use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{ApiResult, LanguageParams},
    database, Difficulty,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "zzz/achievements")),
    paths(get_zzz_achievements),
    components(schemas(Achievement))
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
    timegated: bool,
    missable: bool,
    impossible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    set: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    related: Option<Vec<i32>>,
    percent: f64,
}

impl From<database::zzz::achievements::DbAchievement> for Achievement {
    fn from(db_achievement: database::zzz::achievements::DbAchievement) -> Self {
        Achievement {
            id: db_achievement.id,
            series: db_achievement.series,
            series_name: db_achievement.series_name.clone(),
            name: db_achievement.name.clone(),
            description: db_achievement.description.clone(),
            currency: db_achievement.polychromes,
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
    cfg.service(get_zzz_achievements).configure(id::configure);
}

#[utoipa::path(
    tag = "zzz/achievements",
    get,
    path = "/api/zzz/achievements",
    params(LanguageParams),
    responses(
        (status = 200, description = "[Achievement]", body = Vec<Achievement>),
    )
)]
#[get("/api/zzz/achievements")]
async fn get_zzz_achievements(
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
        database::zzz::achievements::get_all(language_params.lang, &pool).await?;

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
                database::zzz::achievements::get_all_related_ids(achievement.id, set, &pool)
                    .await?,
            );
        }
    }

    Ok(HttpResponse::Ok().json(achievements))
}
