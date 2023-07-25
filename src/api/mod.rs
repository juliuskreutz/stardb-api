use actix_web::web;

mod achievements;
mod characters;
pub mod import;
pub mod mihomo;
pub mod params;
pub mod schemas;
pub mod scores;
pub mod submissions;
pub mod users;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = achievements::openapi();
    openapi.merge(characters::openapi());
    openapi.merge(scores::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievements::configure)
        .configure(characters::configure)
        .configure(scores::configure);
}
