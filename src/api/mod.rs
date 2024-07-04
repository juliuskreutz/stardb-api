mod achievement_series;
mod achievements;
mod book_series;
mod book_series_worlds;
mod books;
mod characters;
//mod community_tier_list;
mod free_jade_alert;
mod import_achievements;
mod import_books;
mod index;
mod languages;
mod light_cones;
mod mihomo;
mod pages;
mod pom_warps_import;
mod scores;
mod select_all;
mod sitemap;
mod skills;
mod srs_warps_import;
mod users;
mod warps;
mod warps_import;
mod zzz;
// mod warps_stats;

use std::env;

use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{guard, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use strum::{Display, EnumString};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    IntoParams, Modify, OpenApi, ToSchema,
};

use crate::{GachaType, Language};

type ApiResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(OpenApi)]
#[openapi(tags((name = "pinned")), components(schemas(Language, GachaType, File)), modifiers(&PrivateAddon))]
struct ApiDoc;

struct PrivateAddon;

impl Modify for PrivateAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("x-api-key"))),
        );
    }
}

#[derive(Deserialize, IntoParams)]
struct LanguageParams {
    #[serde(default)]
    lang: Language,
}

#[derive(Display, EnumString, Serialize, Deserialize, ToSchema, Clone, Copy)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Region {
    Na,
    Eu,
    Asia,
    Cn,
}

#[derive(MultipartForm, ToSchema)]
struct File {
    #[schema(value_type = String, format = Binary)]
    file: TempFile,
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
    //openapi.merge(community_tier_list::openapi());
    openapi.merge(free_jade_alert::openapi());
    openapi.merge(import_achievements::openapi());
    openapi.merge(import_books::openapi());
    openapi.merge(index::openapi());
    openapi.merge(languages::openapi());
    openapi.merge(light_cones::openapi());
    openapi.merge(mihomo::openapi());
    openapi.merge(pages::openapi());
    openapi.merge(pom_warps_import::openapi());
    openapi.merge(scores::openapi());
    openapi.merge(select_all::openapi());
    openapi.merge(sitemap::openapi());
    openapi.merge(srs_warps_import::openapi());
    openapi.merge(skills::openapi());
    openapi.merge(users::openapi());
    openapi.merge(warps::openapi());
    openapi.merge(warps_import::openapi());
    openapi.merge(zzz::openapi());
    // openapi.merge(warps_stats::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig, pool: PgPool) {
    cfg.configure(achievement_series::configure)
        .configure(achievements::configure)
        .configure(book_series::configure)
        .configure(book_series_worlds::configure)
        .configure(books::configure)
        .configure(characters::configure)
        //.configure(community_tier_list::configure)
        .configure(free_jade_alert::configure)
        .configure(import_achievements::configure)
        .configure(import_books::configure)
        .configure(index::configure)
        .configure(languages::configure)
        .configure(light_cones::configure)
        .configure(mihomo::configure)
        .configure(|sc| pages::configure(sc, pool.clone()))
        .configure(pom_warps_import::configure)
        .configure(scores::configure)
        .configure(select_all::configure)
        .configure(|sc| sitemap::configure(sc, pool.clone()))
        .configure(skills::configure)
        .configure(srs_warps_import::configure)
        .configure(users::configure)
        .configure(warps::configure)
        .configure(warps_import::configure)
        .configure(zzz::configure);
    // .configure(warps_stats::configure);
}
