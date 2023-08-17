mod achievements;
mod characters;
mod community_tier_list;
mod free_jade_alert;
mod import;
mod languages;
mod mihomo;
pub mod private;
mod scores;
mod series;
mod users;

use actix_web::web;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};
use utoipa::{IntoParams, OpenApi, ToSchema};

type ApiResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(OpenApi)]
#[openapi(components(schemas(Language)))]
struct ApiDoc;

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

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(achievements::openapi());
    openapi.merge(characters::openapi());
    openapi.merge(community_tier_list::openapi());
    openapi.merge(free_jade_alert::openapi());
    openapi.merge(import::openapi());
    openapi.merge(languages::openapi());
    openapi.merge(mihomo::openapi());
    openapi.merge(scores::openapi());
    openapi.merge(series::openapi());
    openapi.merge(users::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.configure(achievements::configure)
        .configure(characters::configure)
        .configure(community_tier_list::configure)
        .configure(free_jade_alert::configure)
        .configure(import::configure)
        .configure(languages::configure)
        .configure(mihomo::configure)
        .configure(private::configure)
        .configure(scores::configure)
        .configure(series::configure)
        .configure(users::configure);
}
