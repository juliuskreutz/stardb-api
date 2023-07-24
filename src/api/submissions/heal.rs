use actix_session::Session;
use actix_web::{delete, get, post, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    api::{params::*, schemas::*},
    database::{self, DbSubmissionHeal},
    Result,
};

impl<T: AsRef<DbSubmissionHeal>> From<T> for SubmissionHeal {
    fn from(value: T) -> Self {
        let db_submission = value.as_ref();

        Self {
            uuid: db_submission.uuid,
            uid: db_submission.uid,
            heal: db_submission.heal,
            video: db_submission.video.clone(),
            created_at: db_submission.created_at,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/submissions/heal",
    params(
        SubmissionsParams
    ),
    responses(
        (status = 200, description = "[SubmissionHeal]", body = Vec<SubmissionHeal>),
    )
)]
#[get("/api/submissions/heal")]
async fn get_submissions_heal(
    submission_update: web::Query<SubmissionsParams>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let submissions: Vec<_> = database::get_submissions_heal(submission_update.uid, &pool)
        .await?
        .iter()
        .map(SubmissionHeal::from)
        .collect();

    Ok(HttpResponse::Ok().json(submissions))
}

#[utoipa::path(
    get,
    path = "/api/submissions/heal/{uuid}",
    params(
        ("uuid" = String, Path, description = "uuid")
    ),
    responses(
        (status = 200, description = "SubmissionHeal", body = SubmissionHeal),
    )
)]
#[get("/api/submissions/heal/{uuid}")]
async fn get_submission_heal(
    uuid: web::Path<Uuid>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let submission: SubmissionHeal = database::get_submission_heal_by_uuid(*uuid, &pool)
        .await?
        .into();

    Ok(HttpResponse::Ok().json(submission))
}

#[utoipa::path(
    post,
    path = "/api/submissions/heal",
    request_body = SubmissionHealUpdate,
    responses(
        (status = 200, description = "Added submission"),
        (status = 400, description = "Not logged in"),
        (status = 200, description = "Uid not connected to account"),
    )
)]
#[post("/api/submissions/heal")]
async fn post_submission_heal(
    session: Session,
    heal_submission_update: web::Json<SubmissionHealUpdate>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let uid = heal_submission_update.uid;
    let uids = database::get_connections_by_username(&username, &pool).await?;

    if !uids.iter().any(|c| c.uid == uid) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let db_submission_heal = DbSubmissionHeal {
        uid: heal_submission_update.uid,
        heal: heal_submission_update.heal,
        video: heal_submission_update.video.clone(),
        ..Default::default()
    };

    database::set_submission_heal(&db_submission_heal, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    delete,
    path = "/api/submissions/heal/{uuid}",
    params(
        ("uuid" = String, Path, description = "uuid")
    ),
    responses(
        (status = 200, description = "Deleted submission"),
        (status = 403, description = "Not an admin"),
    )
)]
#[delete("/api/submissions/heal/{uuid}")]
async fn delete_submission_heal(
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

    database::delete_submission_heal(*uuid, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}
