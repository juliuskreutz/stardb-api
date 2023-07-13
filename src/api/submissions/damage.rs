use actix_session::Session;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::{params::*, schemas::*},
    database::{self, DbSubmissionDamage},
    Result,
};

impl<T: AsRef<DbSubmissionDamage>> From<T> for SubmissionDamage {
    fn from(value: T) -> Self {
        let db_submission = value.as_ref();

        Self {
            uuid: db_submission.uuid,
            uid: db_submission.uid,
            character: db_submission.character.clone(),
            support: db_submission.support,
            damage: db_submission.damage,
            video: db_submission.video.clone(),
            created_at: db_submission.created_at,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/submissions/damage",
    params(
        SubmissionsParams
    ),
    responses(
        (status = 200, description = "[SubmissionDamage]", body = Vec<SubmissionDamage>),
    )
)]
#[get("/api/submissions/damage")]
async fn get_submissions_damage(
    submission_update: web::Query<SubmissionsParams>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let submissions: Vec<_> = database::get_submissions_damage(submission_update.uid, &pool)
        .await?
        .iter()
        .map(SubmissionDamage::from)
        .collect();

    Ok(HttpResponse::Ok().json(submissions))
}

#[utoipa::path(
    get,
    path = "/api/submissions/damage/{uuid}",
    params(
        ("uuid" = String, Path, description = "uuid")
    ),
    responses(
        (status = 200, description = "SubmissionDamage", body = SubmissionDamage),
    )
)]
#[get("/api/submissions/damage/{uuid}")]
async fn get_submission_damage(
    uuid: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let submission: SubmissionDamage = database::get_submission_damage_by_uuid(*uuid, &pool)
        .await?
        .into();

    Ok(HttpResponse::Ok().json(submission))
}

#[utoipa::path(
    post,
    path = "/api/submissions/damage",
    request_body = SubmissionDamageUpdate,
    responses(
        (status = 200, description = "Added submission"),
    )
)]
#[post("/api/submissions/damage")]
async fn post_submission_damage(
    damage_submission_update: web::Json<SubmissionDamageUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let db_submission_damage = DbSubmissionDamage {
        uid: damage_submission_update.uid,
        character: damage_submission_update.character.to_string(),
        support: damage_submission_update.support,
        damage: damage_submission_update.damage,
        video: damage_submission_update.video.clone(),
        ..Default::default()
    };

    database::set_submission_damage(&db_submission_damage, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    delete,
    path = "/api/submissions/damage/{uuid}",
    params(
        ("uuid" = String, Path, description = "uuid")
    ),
    responses(
        (status = 200, description = "Deleted submission"),
        (status = 403, description = "Not an admin"),
    )
)]
#[delete("/api/submissions/damage/{uuid}")]
async fn delete_submission_damage(
    session: Session,
    uuid: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(admin)) = session.get::<bool>("admin") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !admin {
        return Ok(HttpResponse::Forbidden().finish());
    }

    database::delete_submission_damage(*uuid, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
