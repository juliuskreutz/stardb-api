use actix_web::{delete, get, put, web, HttpRequest, HttpResponse, Responder};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString};
use utoipa::ToSchema;

use crate::api::users::Claims;
use crate::database::{self, DbAchievement, DbComplete};
use crate::Result;

#[derive(Display, EnumString, Serialize, Deserialize, ToSchema)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Diffuculty {
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
    comment: Option<String>,
    reference: Option<String>,
    difficulty: Option<Diffuculty>,
}

impl<T: AsRef<DbAchievement>> From<T> for Achievement {
    fn from(value: T) -> Self {
        let db_achievement = value.as_ref();

        Achievement {
            id: db_achievement.id,
            series: db_achievement.series.clone(),
            title: db_achievement.title.clone(),
            description: db_achievement.description.clone(),
            comment: db_achievement.comment.clone(),
            reference: db_achievement.reference.clone(),
            difficulty: db_achievement
                .difficulty
                .as_ref()
                .map(|d| d.parse().unwrap()),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/achievements",
    responses(
        (status = 200, description = "Achievements", body = Vec<Achievement>),
    )
)]
#[get("/api/achievements")]
async fn get_achievements(pool: web::Data<PgPool>) -> Result<impl Responder> {
    let db_achievements = database::get_achievements(&pool).await?;

    let achievements: Vec<Achievement> = db_achievements.iter().map(Achievement::from).collect();

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
pub struct AchievementUpdate {
    comment: Option<String>,
    reference: Option<String>,
    difficulty: Option<Diffuculty>,
}

#[utoipa::path(
    put,
    path = "/api/achievements/{id}",
    request_body = AchievementUpdate,
    responses(
        (status = 200, description = "Achievement", body = Achievement),
        (status = 403, description = "Not an admin"),
    )
)]
#[put("/api/achievements/{id}")]
async fn put_achievement(
    request: HttpRequest,
    id: web::Path<i64>,
    achievement_update: web::Json<AchievementUpdate>,
    jwt_secret: web::Data<[u8; 32]>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Some(cookie) = request.cookie("token") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let claims: Claims = jsonwebtoken::decode(
        cookie.value(),
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|t| t.claims)?;

    if !claims.admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let db_achievement = DbAchievement {
        id: *id,
        comment: achievement_update.comment.clone(),
        reference: achievement_update.reference.clone(),
        difficulty: achievement_update
            .difficulty
            .as_ref()
            .map(|d| d.to_string()),
        ..Default::default()
    };

    let achievement: Achievement = database::set_achievement(&db_achievement, &pool)
        .await?
        .into();

    Ok(HttpResponse::Ok().json(achievement))
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
async fn get_completed(
    request: HttpRequest,
    jwt_secret: web::Data<[u8; 32]>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Some(cookie) = request.cookie("token") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let claims: Claims = jsonwebtoken::decode(
        cookie.value(),
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|t| t.claims)?;

    let completed: Vec<_> = database::get_completed_by_username(&claims.username, &pool)
        .await?
        .iter()
        .map(|c| c.achievement)
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
    request: HttpRequest,
    id: web::Path<i64>,
    jwt_secret: web::Data<[u8; 32]>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Some(cookie) = request.cookie("token") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let claims: Claims = jsonwebtoken::decode(
        cookie.value(),
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|t| t.claims)?;

    let achievement = *id;
    let username = claims.username.clone();

    let db_complete = DbComplete {
        username,
        achievement,
    };

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
    request: HttpRequest,
    id: web::Path<i64>,
    jwt_secret: web::Data<[u8; 32]>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Some(cookie) = request.cookie("token") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let claims: Claims = jsonwebtoken::decode(
        cookie.value(),
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map(|t| t.claims)?;

    let achievement = *id;
    let username = claims.username.clone();

    let db_complete = DbComplete {
        username,
        achievement,
    };

    database::delete_complete(&db_complete, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
