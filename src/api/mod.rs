mod achievement_series;
mod achievements;
mod book_series;
mod book_series_worlds;
mod books;
mod characters;
mod community_tier_list;
mod free_jade_alert;
mod import;
mod languages;
mod mihomo;
mod pages;
mod scores;
mod users;

use std::env;

use actix_web::{guard, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumIter, EnumString};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    IntoParams, Modify, OpenApi, ToSchema,
};

type ApiResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(OpenApi)]
#[openapi(components(schemas(Language)), modifiers(&PrivateAddon))]
struct ApiDoc;

struct PrivateAddon;

impl Modify for PrivateAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
        )
    }
}

#[derive(Deserialize, IntoParams)]
struct LanguageParams {
    #[serde(default)]
    lang: Language,
}

#[derive(
    Default,
    PartialEq,
    Eq,
    Hash,
    Display,
    EnumString,
    EnumIter,
    Serialize,
    Deserialize,
    ToSchema,
    Clone,
    Copy,
)]
#[strum(serialize_all = "lowercase")]
#[serde(rename_all = "lowercase")]
enum Language {
    Chs,
    Cht,
    De,
    #[default]
    En,
    Es,
    Fr,
    Id,
    Jp,
    Kr,
    Pt,
    Ru,
    Th,
    Vi,
}

impl Language {
    pub fn name(&self) -> String {
        match self {
            Language::Chs => "简体中文",
            Language::Cht => "繁體中文",
            Language::De => "Deutsch",
            Language::En => "English",
            Language::Es => "Español",
            Language::Fr => "Français",
            Language::Id => "Bahasa Indonesia",
            Language::Jp => "日本語",
            Language::Kr => "한국어",
            Language::Pt => "Português",
            Language::Ru => "Русский",
            Language::Th => "ไทย",
            Language::Vi => "Tiếng Việt",
        }
        .to_string()
    }
}

fn private(ctx: &guard::GuardContext) -> bool {
    Some(env::var("API_KEY").unwrap().as_bytes())
        == ctx.head().headers().get("x-api-key").map(|h| h.as_bytes())
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievement_series::openapi());
    openapi.merge(achievements::openapi());
    openapi.merge(book_series::openapi());
    openapi.merge(book_series_worlds::openapi());
    openapi.merge(books::openapi());
    openapi.merge(characters::openapi());
    openapi.merge(community_tier_list::openapi());
    openapi.merge(free_jade_alert::openapi());
    openapi.merge(import::openapi());
    openapi.merge(languages::openapi());
    openapi.merge(mihomo::openapi());
    openapi.merge(pages::openapi());
    openapi.merge(scores::openapi());
    openapi.merge(users::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievement_series::configure)
        .configure(achievements::configure)
        .configure(book_series::configure)
        .configure(book_series_worlds::configure)
        .configure(books::configure)
        .configure(characters::configure)
        .configure(community_tier_list::configure)
        .configure(free_jade_alert::configure)
        .configure(import::configure)
        .configure(languages::configure)
        .configure(mihomo::configure)
        .configure(pages::configure)
        .configure(scores::configure)
        .configure(users::configure);
}

pub fn cache_achievement_tracker(
    pool: PgPool,
) -> web::Data<pages::achievement_tracker::AchievementTrackerCache> {
    pages::cache_achievement_tracker(pool)
}
