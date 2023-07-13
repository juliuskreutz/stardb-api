use actix_session::Session;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::{params::*, schemas::*},
    database::{self, DbSubmissionShield},
    Result,
};

impl<T: AsRef<DbSubmissionShield>> From<T> for SubmissionShield {
    fn from(value: T) -> Self {
        let db_submission = value.as_ref();

        Self {
            uuid: db_submission.uuid,
            uid: db_submission.uid,
            shield: db_submission.shield,
            video: db_submission.video.clone(),
            created_at: db_submission.created_at,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/submissions/shield",
    params(
        SubmissionsParams
    ),
    responses(
        (status = 200, description = "[SubmissionShield]", body = Vec<SubmissionShield>),
    )
)]
#[get("/api/submissions/shield")]
async fn get_submissions_shield(
    submission_update: web::Query<SubmissionsParams>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let submissions: Vec<_> = database::get_submissions_shield(submission_update.uid, &pool)
        .await?
        .iter()
        .map(SubmissionShield::from)
        .collect();

    Ok(HttpResponse::Ok().json(submissions))
}

#[utoipa::path(
    get,
    path = "/api/submissions/shield/{uuid}",
    params(
        ("uuid" = String, Path, description = "uuid")
    ),
    responses(
        (status = 200, description = "SubmissionShield", body = SubmissionShield),
    )
)]
#[get("/api/submissions/shield/{uuid}")]
async fn get_submission_shield(
    uuid: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let submission: SubmissionShield = database::get_submission_shield_by_uuid(*uuid, &pool)
        .await?
        .into();

    Ok(HttpResponse::Ok().json(submission))
}

#[utoipa::path(
    post,
    path = "/api/submissions/shield",
    request_body = SubmissionShieldUpdate,
    responses(
        (status = 200, description = "Added submission"),
    )
)]
#[post("/api/submissions/shield")]
async fn post_submission_shield(
    shield_submission_update: web::Json<SubmissionShieldUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let db_submission_shield = DbSubmissionShield {
        uid: shield_submission_update.uid,
        shield: shield_submission_update.shield,
        video: shield_submission_update.video.clone(),
        ..Default::default()
    };

    database::set_submission_shield(&db_submission_shield, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    delete,
    path = "/api/submissions/shield/{uuid}",
    params(
        ("uuid" = String, Path, description = "uuid")
    ),
    responses(
        (status = 200, description = "Deleted submission"),
        (status = 403, description = "Not an admin"),
    )
)]
#[delete("/api/submissions/shield/{uuid}")]
async fn delete_submission_shield(
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

    database::delete_submission_shield(*uuid, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
