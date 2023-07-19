use actix_session::Session;
use actix_web::{delete, get, put, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString};
use utoipa::ToSchema;

use crate::database::{self, DbAchievement, DbComplete};
use crate::Result;

#[derive(Display, EnumString, Serialize, Deserialize, ToSchema)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Serialize, ToSchema)]
pub struct Achievement {
    id: i64,
    series: String,
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
            series: db_achievement.series.clone(),
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

#[utoipa::path(
    get,
    path = "/api/achievements",
    responses(
        (status = 200, description = "Achievements", body = Vec<Vec<Achievement>>),
    )
)]
#[get("/api/achievements")]
async fn get_achievements(pool: web::Data<PgPool>) -> Result<impl Responder> {
    let db_achievements = database::get_achievements(&pool).await?;

    let mut last_grouping = None;
    let mut achievements: Vec<Vec<Achievement>> = Vec::new();

    for db_achievement in db_achievements {
        if let Some(grouping) = db_achievement.grouping {
            let achievement = db_achievement.into();

            if last_grouping == Some(grouping) {
                let i = achievements.len() - 1;
                achievements[i].push(achievement)
            } else {
                last_grouping = Some(grouping);
                achievements.push(vec![achievement]);
            }
        } else {
            last_grouping = None;
            achievements.push(vec![db_achievement.into()]);
        }
    }

    Ok(HttpResponse::Ok().json(achievements))
}

#[utoipa::path(
    get,
    path = "/api/achievements/{id}",
    responses(
        (status = 200, description = "Achievement", body = Achievement),
    )
)]
#[get("/api/achievements/{id}")]
async fn get_achievement(id: web::Path<i64>, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let achievement: Achievement = database::get_achievement_by_id(*id, &pool).await?.into();

    Ok(HttpResponse::Ok().json(achievement))
}

#[derive(Deserialize, ToSchema)]
pub struct CommentUpdate {
    comment: String,
}

#[utoipa::path(
    put,
    path = "/api/achievements/{id}/comment",
    request_body = CommentUpdate,
    responses(
        (status = 200, description = "Updated comment"),
        (status = 403, description = "Not an admin"),
    )
)]
#[put("/api/achievements/{id}/comment")]
async fn put_achievement_comment(
    session: Session,
    id: web::Path<i64>,
    comment_update: web::Json<CommentUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::update_achievement_comment(*id, &comment_update.comment, &pool).await?;

    Ok(HttpResponse::Ok().finish())
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
    path = "/api/achievements/{id}/comment",
    responses(
        (status = 200, description = "Deleted comment"),
        (status = 403, description = "Not an admin"),
    )
)]
#[delete("/api/achievements/{id}/comment")]
async fn delete_achievement_comment(
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

    database::delete_achievement_comment(*id, &pool).await?;

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

#[utoipa::path(
    get,
    path = "/api/achievements/completed",
    responses(
        (status = 200, description = "Achievement ids", body = Vec<i64>),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/achievements/completed")]
async fn get_completed(session: Session, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let completed: Vec<_> = database::get_completed_by_username(&username, &pool)
        .await?
        .iter()
        .map(|c| c.id)
        .collect();

    Ok(HttpResponse::Ok().json(completed))
}

#[utoipa::path(
    put,
    path = "/api/achievements/completed/{id}",
    responses(
        (status = 200, description = "Successful add of the achievement."),
        (status = 400, description = "Not logged in"),
    )
)]
#[put("/api/achievements/completed/{id}")]
async fn put_complete(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let db_complete = DbComplete { username, id };

    database::add_complete(&db_complete, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    delete,
    path = "/api/achievements/completed/{id}",
    responses(
        (status = 200, description = "Successful delete of the achievement."),
        (status = 400, description = "Not logged in"),
    )
)]
#[delete("/api/achievements/completed/{id}")]
async fn delete_complete(
    session: Session,
    id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let id = *id;

    let db_complete = DbComplete { username, id };

    database::delete_complete(&db_complete, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
