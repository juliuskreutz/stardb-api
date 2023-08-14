use actix_session::Session;
use actix_web::{post, web, HttpResponse, Responder};
use rand::Rng;
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{
    database::{self, DbUser},
    Result,
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
    responses(
        (status = 200, description = "Successfull register. The session id is returned in a cookie named `id`. You need to include this cookie in subsequent requests"),
        (status = 400, description = "Credentials too long"),
        (status = 409, description = "Account already exists")
    )
)]
#[post("/api/users/auth/register")]
async fn register(
    session: Session,
    user_register: web::Json<UserRegister>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let username = user_register.username.clone();
    let password = user_register.password.clone();
    let email = user_register.email.clone();

    if username.len() > 32
        || password.len() > 64
        || email.as_ref().map(|s| s.len()).unwrap_or_default() > 64
    {
        return Ok(HttpResponse::BadRequest().finish());
    }

    if database::get_user_by_username(&username, &pool)
        .await
        .is_ok()
    {
        return Ok(HttpResponse::Conflict().finish());
    }

    let salt = rand::thread_rng().gen::<[u8; 32]>();

    let password = argon2::hash_encoded(
        password.as_bytes(),
        &salt,
        &argon2::Config::rfc9106_low_mem(),
    )?;

    {
        let username = username.clone();
        let user = DbUser {
            username,
            password,
            email,
            admin: false,
        };
        database::set_user(&user, &pool).await?;
    }

    session.insert("username", username)?;
    session.insert("admin", false)?;

    Ok(HttpResponse::Ok().finish())
}
