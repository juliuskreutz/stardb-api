use actix_session::Session;
use actix_web::{delete, put, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString};
use utoipa::{OpenApi, ToSchema};

mod get;
mod id;

use crate::database::{self, DbAchievement};
use crate::Result;

#[derive(OpenApi)]
#[openapi(components(schemas(Difficulty, Achievement)), tags((name = "Achievements", description = "All the important achievements methods")))]
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
    series_name: String,
    title: String,
    description: String,
    jades: i32,
    hidden: bool,
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
    openapi.merge(get::openapi());
    openapi.merge(id::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(get::configure).configure(id::configure);
}

#[derive(Deserialize, ToSchema)]
pub struct ReferenceUpdate {
    reference: String,
}

#[utoipa::path(
    put,
    path = "/api/achievements/{id}/reference",
    request_body = ReferenceUpdate,
    responses(
        (status = 200, description = "Updated reference"),
        (status = 403, description = "Not an admin"),
    )
)]
#[put("/api/achievements/{id}/reference")]
async fn put_achievement_reference(
    session: Session,
    id: web::Path<i64>,
    reference_update: web::Json<ReferenceUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::update_achievement_reference(*id, &reference_update.reference, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, ToSchema)]
pub struct DifficultyUpdate {
    difficulty: Difficulty,
}

#[utoipa::path(
    put,
    path = "/api/achievements/{id}/difficulty",
    request_body = DifficultyUpdate,
    responses(
        (status = 200, description = "Updated difficulty"),
        (status = 403, description = "Not an admin"),
    )
)]
#[put("/api/achievements/{id}/difficulty")]
async fn put_achievement_difficulty(
    session: Session,
    id: web::Path<i64>,
    difficulty_update: web::Json<DifficultyUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::update_achievement_difficulty(*id, &difficulty_update.difficulty.to_string(), &pool)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    delete,
    path = "/api/achievements/{id}/reference",
    responses(
        (status = 200, description = "Deleted reference"),
        (status = 403, description = "Not an admin"),
    )
)]
#[delete("/api/achievements/{id}/reference")]
async fn delete_achievement_reference(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::delete_achievement_reference(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    delete,
    path = "/api/achievements/{id}/difficulty",
    responses(
        (status = 200, description = "Deleted difficulty"),
        (status = 403, description = "Not an admin"),
    )
)]
#[delete("/api/achievements/{id}/difficulty")]
async fn delete_achievement_difficulty(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::delete_achievement_difficulty(*id, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
