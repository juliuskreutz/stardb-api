use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use rand::Rng;
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    api::{private, ApiResult},
    database,
};

#[derive(OpenApi)]
#[openapi(
    tags((name = "users/auth/register")),
    paths(register),
    components(schemas(UserRegister))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
}

#[derive(Deserialize, ToSchema)]
pub struct UserRegister {
    username: String,
    password: String,
    email: Option<String>,
}

#[utoipa::path(
    tag = "users/auth/register",
    post,
    path = "/api/users/auth/register",
    request_body(content = UserRegister,
        examples(
            ("UsernamePassword" = (value = json!({"username": "meow", "password": "meow12"}))),
            ("UsernamePasswordEmail" = (value = json!({"username": "meow", "password": "meow12", "email": "meow@gmail.com"})))
        )
    ),
    security(
        ("api_key" = [])
    ),
    responses(
        (status = 200, description = "Successfull register. The session id is returned in a cookie named `id`. You need to include this cookie in subsequent requests"),
        (status = 400, description = "Credentials too long"),
        (status = 409, description = "Account already exists")
    )
)]
#[post("/api/users/auth/register", guard = "private")]
async fn register(
    session: Session,
    user_register: web::Json<UserRegister>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let username = user_register.username.trim().to_lowercase();
    let password = user_register.password.clone();
    let email = user_register.email.clone();

    if username.len() > 32
        || password.len() > 64
        || email.as_ref().map(|s| s.len()).unwrap_or_default() > 64
    {
        return Ok(HttpResponse::BadRequest().finish());
    }

    if database::users::get_one_by_username(&username, &pool)
        .await
        .is_ok()
    {
        return Ok(HttpResponse::Conflict().finish());
    }

    let salt = rand::rng().random::<[u8; 32]>();

    let password = argon2::hash_encoded(
        password.as_bytes(),
        &salt,
        &argon2::Config::rfc9106_low_mem(),
    )?;

    {
        let username = username.clone();
        let user = database::users::DbUser {
            username,
            password,
            email,
        };
        database::users::set(&user, &pool).await?;
    }

    session.insert("username", username)?;

    Ok(HttpResponse::Ok().finish())
}
