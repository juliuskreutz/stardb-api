mod achievement_series;
mod achievements;
mod admin;
mod banner_helpers;
mod banners;
mod characters;
mod gi;
mod import_achievements;
mod import_gi_achievements;
mod import_jobs;
mod import_zzz_achievements;
mod languages;
mod light_cones;
mod mihomo;
mod ntehelper;
mod pages;
mod pom_warps_import;
mod scores;
mod select_all;
mod sitemap;
mod srgf_warps_import;
mod srs_warps_import;
mod uigf_import;
mod users;
mod warps;
mod warps_import;
mod zzz;

use std::env;

use crate::app_config::AppConfig;
use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{guard, web, HttpResponse};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use strum::{Display, EnumString};
use url::Url;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    IntoParams, Modify, OpenApi, ToSchema,
};

use crate::{Difficulty, GachaType, GiGachaType, Language, ZzzGachaType};

type ApiResult<T> = Result<T, Box<dyn std::error::Error>>;

pub(crate) fn gacha_history_forbidden(
    is_private: bool,
    authenticated: bool,
    is_admin: bool,
    has_verified_connection: bool,
) -> bool {
    is_private && !(authenticated && (is_admin || has_verified_connection))
}

pub(crate) fn validate_import_url(raw_url: &str) -> Result<Url, HttpResponse> {
    let Ok(url) = Url::parse(raw_url) else {
        return Err(HttpResponse::BadRequest().body("Invalid URL"));
    };

    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(HttpResponse::BadRequest().body("Invalid URL"));
    }

    Ok(url)
}

#[derive(OpenApi)]
#[openapi(tags((name = "pinned")), components(schemas(Language, GachaType, ZzzGachaType, GiGachaType, File, Difficulty)), modifiers(&PrivateAddon))]
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
    if cfg!(debug_assertions) {
        return true;
    }

    Some(env::var("API_KEY").unwrap().as_bytes())
        == ctx.head().headers().get("x-api-key").map(|h| h.as_bytes())
}

pub fn openapi() -> utoipa::openapi::OpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(admin::openapi());
    openapi.merge(achievement_series::openapi());
    openapi.merge(achievements::openapi());
    openapi.merge(banners::openapi());
    openapi.merge(characters::openapi());
    openapi.merge(gi::openapi());
    openapi.merge(import_achievements::openapi());
    openapi.merge(import_gi_achievements::openapi());
    openapi.merge(import_zzz_achievements::openapi());
    openapi.merge(languages::openapi());
    openapi.merge(light_cones::openapi());
    openapi.merge(mihomo::openapi());
    openapi.merge(ntehelper::openapi());
    openapi.merge(pages::openapi());
    openapi.merge(pom_warps_import::openapi());
    openapi.merge(scores::openapi());
    openapi.merge(select_all::openapi());
    openapi.merge(sitemap::openapi());
    openapi.merge(srgf_warps_import::openapi());
    openapi.merge(srs_warps_import::openapi());
    openapi.merge(uigf_import::openapi());
    openapi.merge(users::openapi());
    openapi.merge(warps::openapi());
    openapi.merge(warps_import::openapi());
    openapi.merge(zzz::openapi());
    openapi
}

pub fn configure(
    cfg: &mut web::ServiceConfig,
    pool: PgPool,
    app_config: web::Data<Arc<AppConfig>>,
) {
    cfg.configure(admin::configure)
        .configure(achievement_series::configure)
        .configure(achievements::configure)
        .configure(banners::configure)
        .configure(characters::configure)
        .configure(gi::configure)
        .configure(import_achievements::configure)
        .configure(import_gi_achievements::configure)
        .configure(import_zzz_achievements::configure)
        .configure(languages::configure)
        .configure(light_cones::configure)
        .configure(mihomo::configure)
        .configure(ntehelper::configure)
        .configure(|sc| pages::configure(sc, pool.clone(), app_config.clone()))
        .configure(pom_warps_import::configure)
        .configure(scores::configure)
        .configure(select_all::configure)
        .configure(|sc| sitemap::configure(sc, pool.clone(), app_config.clone()))
        .configure(srgf_warps_import::configure)
        .configure(srs_warps_import::configure)
        .configure(uigf_import::configure)
        .configure(users::configure)
        .configure(warps::configure)
        .configure(warps_import::configure)
        .configure(zzz::configure);
}

#[cfg(test)]
mod gacha_security {
    mod privacy {
        use super::super::gacha_history_forbidden;
        use actix_session::{Session, SessionMiddleware};
        use actix_web::{
            cookie::{Cookie, Key},
            http::StatusCode,
            test as aw_test, web, App, HttpResponse,
        };
        use sqlx::{postgres::PgPoolOptions, PgPool};
        use uuid::Uuid;

        async fn test_pool() -> PgPool {
            let database_url =
                std::env::var("DATABASE_URL").expect("DATABASE_URL is required for DB tests");
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .expect("test database connects");
            sqlx::migrate!()
                .run(&pool)
                .await
                .expect("test database is migrated");
            pool
        }

        async fn test_login(session: Session, username: web::Path<String>) -> HttpResponse {
            session
                .insert("username", username.into_inner())
                .expect("test session stores username");
            HttpResponse::NoContent().finish()
        }

        async fn insert_user(pool: &PgPool, username: &str) {
            sqlx::query("INSERT INTO users (username, password) VALUES ($1, 'test')")
                .bind(username)
                .execute(pool)
                .await
                .expect("test user inserts");
        }

        #[test]
        fn access_matrix_is_explicit() {
            assert!(!gacha_history_forbidden(false, false, false, false));
            assert!(gacha_history_forbidden(true, false, false, false));
            assert!(gacha_history_forbidden(true, true, false, false));
            assert!(!gacha_history_forbidden(true, true, false, true));
            assert!(!gacha_history_forbidden(true, true, true, false));
        }

        #[actix_web::test]
        async fn game_specific_connections_enforce_the_full_access_matrix() {
            let pool = test_pool().await;
            let suffix = Uuid::new_v4().simple().to_string();
            let owner = format!("gacha_owner_{suffix}");
            let unverified = format!("gacha_unverified_{suffix}");
            let admin = format!("gacha_admin_{suffix}");
            let hsr_owner = format!("gacha_hsr_{suffix}");
            let uid = 1_500_000_000 + (Uuid::new_v4().as_u128() % 100_000_000) as i32;

            for username in [&owner, &unverified, &admin, &hsr_owner] {
                insert_user(&pool, username).await;
            }
            sqlx::query("INSERT INTO admins (username) VALUES ($1)")
                .bind(&admin)
                .execute(&pool)
                .await
                .expect("test admin inserts");
            sqlx::query(
                "INSERT INTO mihomo (uid, region, name, level, signature, avatar_icon, achievement_count) VALUES ($1, 'na', 'test', 1, '', '', 0)",
            )
            .bind(uid)
            .execute(&pool)
            .await
            .expect("HSR profile inserts");
            sqlx::query("INSERT INTO gi_profiles (uid, name) VALUES ($1, 'test')")
                .bind(uid)
                .execute(&pool)
                .await
                .expect("GI profile inserts");
            sqlx::query("INSERT INTO zzz_uids (uid) VALUES ($1)")
                .bind(uid)
                .execute(&pool)
                .await
                .expect("ZZZ UID inserts");
            sqlx::query(
                "INSERT INTO connections (uid, username, verified, private) VALUES ($1, $2, true, true)",
            )
            .bind(uid)
            .bind(&hsr_owner)
            .execute(&pool)
            .await
            .expect("private HSR connection inserts");

            let app = aw_test::init_service(
                App::new()
                    .app_data(web::Data::new(pool.clone()))
                    .wrap(
                        SessionMiddleware::builder(
                            crate::pg_session_store::PgSessionStore::new(pool.clone()),
                            Key::generate(),
                        )
                        .cookie_secure(false)
                        .build(),
                    )
                    .route("/test-login/{username}", web::post().to(test_login))
                    .configure(super::super::gi::configure)
                    .configure(super::super::zzz::configure),
            )
            .await;

            for uri in [
                format!("/api/gi/wishes/{uid}"),
                format!("/api/zzz/signals/{uid}"),
            ] {
                let response =
                    aw_test::call_service(&app, aw_test::TestRequest::get().uri(&uri).to_request())
                        .await;
                assert_eq!(
                    response.status(),
                    StatusCode::OK,
                    "HSR privacy leaked into {uri}"
                );
            }

            for table in ["gi_connections", "zzz_connections"] {
                sqlx::query(&format!(
                    "INSERT INTO {table} (uid, username, verified, private) VALUES ($1, $2, true, true), ($1, $3, false, false)"
                ))
                .bind(uid)
                .bind(&owner)
                .bind(&unverified)
                .execute(&pool)
                .await
                .expect("game connections insert");
            }

            let owner_login = aw_test::call_service(
                &app,
                aw_test::TestRequest::post()
                    .uri(&format!("/test-login/{owner}"))
                    .to_request(),
            )
            .await;
            assert_eq!(owner_login.status(), StatusCode::NO_CONTENT);
            let owner_cookie: Cookie<'static> = owner_login
                .response()
                .cookies()
                .next()
                .expect("owner session cookie is returned")
                .into_owned();

            let unverified_login = aw_test::call_service(
                &app,
                aw_test::TestRequest::post()
                    .uri(&format!("/test-login/{unverified}"))
                    .to_request(),
            )
            .await;
            assert_eq!(unverified_login.status(), StatusCode::NO_CONTENT);
            let unverified_cookie: Cookie<'static> = unverified_login
                .response()
                .cookies()
                .next()
                .expect("unverified session cookie is returned")
                .into_owned();

            let admin_login = aw_test::call_service(
                &app,
                aw_test::TestRequest::post()
                    .uri(&format!("/test-login/{admin}"))
                    .to_request(),
            )
            .await;
            assert_eq!(admin_login.status(), StatusCode::NO_CONTENT);
            let admin_cookie: Cookie<'static> = admin_login
                .response()
                .cookies()
                .next()
                .expect("admin session cookie is returned")
                .into_owned();

            for uri in [
                format!("/api/gi/wishes/{uid}"),
                format!("/api/zzz/signals/{uid}"),
            ] {
                let anonymous =
                    aw_test::call_service(&app, aw_test::TestRequest::get().uri(&uri).to_request())
                        .await;
                assert_eq!(anonymous.status(), StatusCode::FORBIDDEN);

                let unverified_response = aw_test::call_service(
                    &app,
                    aw_test::TestRequest::get()
                        .uri(&uri)
                        .cookie(unverified_cookie.clone())
                        .to_request(),
                )
                .await;
                assert_eq!(unverified_response.status(), StatusCode::FORBIDDEN);

                let owner_response = aw_test::call_service(
                    &app,
                    aw_test::TestRequest::get()
                        .uri(&uri)
                        .cookie(owner_cookie.clone())
                        .to_request(),
                )
                .await;
                assert_eq!(owner_response.status(), StatusCode::OK);

                let admin_response = aw_test::call_service(
                    &app,
                    aw_test::TestRequest::get()
                        .uri(&uri)
                        .cookie(admin_cookie.clone())
                        .to_request(),
                )
                .await;
                assert_eq!(admin_response.status(), StatusCode::OK);
            }

            for username in [&owner, &unverified, &admin, &hsr_owner] {
                sqlx::query("DELETE FROM users WHERE username = $1")
                    .bind(username)
                    .execute(&pool)
                    .await
                    .expect("test user cleanup succeeds");
            }
            sqlx::query("DELETE FROM gi_profiles WHERE uid = $1")
                .bind(uid)
                .execute(&pool)
                .await
                .expect("GI profile cleanup succeeds");
            sqlx::query("DELETE FROM zzz_uids WHERE uid = $1")
                .bind(uid)
                .execute(&pool)
                .await
                .expect("ZZZ UID cleanup succeeds");
            sqlx::query("DELETE FROM mihomo WHERE uid = $1")
                .bind(uid)
                .execute(&pool)
                .await
                .expect("HSR profile cleanup succeeds");
        }
    }
}
