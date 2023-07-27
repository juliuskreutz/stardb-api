use actix_web::web;

mod achievements;
mod characters;
pub mod import;
pub mod mihomo;
pub mod params;
pub mod schemas;
mod scores;
mod series;
pub mod submissions;
pub mod users;

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = achievements::openapi();
    openapi.merge(series::openapi());
    openapi.merge(characters::openapi());
    openapi.merge(scores::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievements::configure)
        .configure(series::configure)
        .configure(characters::configure)
        .configure(scores::configure);
}
