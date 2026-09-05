//! Opaque in-memory job tracking shared by long-running gacha import endpoints.
//!
//! Clients receive random IDs and closed status/error enums. Upstream request
//! URLs, including credential-bearing query strings, never enter responses.

use std::{collections::HashMap, sync::Arc};

use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Unpredictable public identifier for one import job.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct ImportJobId(Uuid);

impl ImportJobId {
    /// Creates a cryptographically random job identifier.
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for ImportJobId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Stable client-facing categories for import failures.
#[derive(Clone, Copy, Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImportErrorCode {
    UpstreamUnavailable,
    InvalidResponse,
    PersistenceFailed,
}

/// Maps transport and decoding failures to closed public error codes.
pub(crate) fn classify_import_error(
    error: &(dyn std::error::Error + 'static),
    fallback: ImportErrorCode,
) -> ImportErrorCode {
    if error.is::<InvalidImportResponse>() {
        return ImportErrorCode::InvalidResponse;
    }
    match error.downcast_ref::<reqwest::Error>() {
        Some(error) if error.is_decode() => ImportErrorCode::InvalidResponse,
        Some(_) => ImportErrorCode::UpstreamUnavailable,
        None => fallback,
    }
}

/// Removes request URLs from HTTP errors before internal logging.
pub(crate) fn redacted_error(error: Box<dyn std::error::Error>) -> String {
    match error.downcast::<reqwest::Error>() {
        Ok(error) => error.without_url().to_string(),
        Err(error) => error.to_string(),
    }
}

/// Public lifecycle state returned by import job status routes.
#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImportStatus {
    Pending,
    Finished,
    Error(ImportErrorCode),
}

struct ImportJob<T> {
    uid: Option<i32>,
    finished: bool,
    info: Arc<Mutex<T>>,
}

/// Result of creating or joining an active import job.
pub(crate) struct StartedImportJob<T> {
    pub(crate) id: ImportJobId,
    pub(crate) info: Arc<Mutex<T>>,
    pub(crate) is_new: bool,
}

/// Concurrent job store with at most one UID-bound job active per UID.
pub(crate) struct ImportJobStore<T> {
    jobs: Arc<Mutex<HashMap<ImportJobId, ImportJob<T>>>>,
}

impl<T> Default for ImportJobStore<T> {
    fn default() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<T: Send + 'static> ImportJobStore<T> {
    /// Starts a UID-bound job or joins the existing active job for that UID.
    pub(crate) async fn start(&self, uid: i32, initial: T) -> StartedImportJob<T> {
        // Keep lookup and insertion under the same lock: concurrent starts must
        // share unfinished work, while retained completed jobs remain pollable.
        let mut jobs = self.jobs.lock().await;

        if let Some((id, job)) = jobs
            .iter()
            .find(|(_, job)| job.uid == Some(uid) && !job.finished)
        {
            return StartedImportJob {
                id: *id,
                info: job.info.clone(),
                is_new: false,
            };
        }

        let id = ImportJobId::new();
        let info = Arc::new(Mutex::new(initial));
        jobs.insert(
            id,
            ImportJob {
                uid: Some(uid),
                finished: false,
                info: info.clone(),
            },
        );

        StartedImportJob {
            id,
            info,
            is_new: true,
        }
    }

    /// Creates an unbound job for workflows that do not know a UID up front.
    pub(crate) async fn create(&self, initial: T) -> StartedImportJob<T> {
        let mut jobs = self.jobs.lock().await;
        let id = ImportJobId::new();
        let info = Arc::new(Mutex::new(initial));
        jobs.insert(
            id,
            ImportJob {
                uid: None,
                finished: false,
                info: info.clone(),
            },
        );

        StartedImportJob {
            id,
            info,
            is_new: true,
        }
    }

    /// Returns shared mutable job state for an exact opaque identifier.
    pub(crate) async fn get(&self, id: ImportJobId) -> Option<Arc<Mutex<T>>> {
        self.jobs.lock().await.get(&id).map(|job| job.info.clone())
    }

    /// Marks store membership complete and retains the job for 60 seconds from this first call.
    /// Does not mutate T's public status; callers set it before completion. Repeats
    /// neither extend retention nor prevent a new job from starting for the UID.
    pub(crate) async fn complete(&self, id: ImportJobId) {
        self.complete_with_retention(id, std::time::Duration::from_secs(60))
            .await;
    }
    /// Marks a job finished under the store lock and schedules eviction by opaque ID.
    /// Repeated completion is a no-op, so it cannot extend retention or remove a newer UID job.
    async fn complete_with_retention(&self, id: ImportJobId, retention: std::time::Duration) {
        let mut jobs = self.jobs.lock().await;
        let Some(job) = jobs.get_mut(&id) else {
            return;
        };
        // Mark completion while locked, before scheduling eviction. Repeated
        // completions cannot start another timer or extend the retention window.
        if job.finished {
            return;
        }
        job.finished = true;
        let jobs = self.jobs.clone();
        actix_web::rt::spawn(async move {
            actix_web::rt::time::sleep(retention).await;
            // Evict by opaque ID, never UID: a new import may already be active.
            jobs.lock().await.remove(&id);
        });
    }
    #[cfg(test)]
    pub(crate) async fn remove(&self, id: ImportJobId) {
        self.jobs.lock().await.remove(&id);
    }
}

#[cfg(test)]
mod gacha_security {
    mod import_jobs {
        use super::super::{
            classify_import_error, redacted_error, ImportErrorCode, ImportJobId, ImportJobStore,
            ImportStatus,
        };

        #[actix_web::test]
        async fn concurrent_starts_share_one_active_job() {
            let store = ImportJobStore::default();
            let (first, second) =
                futures::join!(store.start(123, "first"), store.start(123, "second"));

            assert_ne!(first.is_new, second.is_new);
            assert_eq!(first.id, second.id);
            let first_info = *first.info.lock().await;
            let second_info = *second.info.lock().await;
            assert_eq!(first_info, second_info);
        }

        #[actix_web::test]
        async fn remove_clears_both_indices() {
            let store = ImportJobStore::default();
            let first = store.start(123, "first").await;

            store.remove(first.id).await;

            assert!(store.get(first.id).await.is_none());
            let second = store.start(123, "second").await;
            assert!(second.is_new);
            assert_ne!(first.id, second.id);
        }

        #[actix_web::test]
        async fn random_uuid_cannot_retrieve_a_job() {
            use actix_web::{http::StatusCode, test, App};

            let app =
                test::init_service(App::new().configure(crate::api::warps_import::configure)).await;
            let response = test::call_service(
                &app,
                test::TestRequest::get()
                    .uri(&format!("/api/warps-import/jobs/{}", ImportJobId::new()))
                    .to_request(),
            )
            .await;

            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        #[test]
        fn public_error_status_contains_only_a_closed_code() {
            let status = ImportStatus::Error(ImportErrorCode::UpstreamUnavailable);
            let json = serde_json::to_string(&status).expect("status serializes");

            assert_eq!(json, r#"{"error":"upstream_unavailable"}"#);
            assert!(!json.contains("authkey"));
            assert!(!json.contains("http"));
        }

        #[actix_web::test]
        async fn reqwest_errors_are_classified_and_have_urls_removed() {
            let error = reqwest::get("http://127.0.0.1:0/?authkey=do-not-expose")
                .await
                .expect_err("port zero cannot accept requests");

            assert!(matches!(
                classify_import_error(&error, ImportErrorCode::PersistenceFailed),
                ImportErrorCode::UpstreamUnavailable
            ));

            let message = redacted_error(Box::new(error));
            assert!(!message.contains("authkey"));
            assert!(!message.contains("do-not-expose"));
            assert!(!message.contains("http://"));
        }

        #[test]
        fn openapi_exposes_only_opaque_status_routes() {
            let openapi = crate::api::openapi();
            let paths = &openapi.paths.paths;

            for path in [
                "/api/warps-import/jobs/{job_id}",
                "/api/gi/wishes-import/jobs/{job_id}",
                "/api/zzz/signals-import/jobs/{job_id}",
            ] {
                assert!(paths.contains_key(path), "missing {path}");
            }

            for path in [
                "/api/warps-import/{uid}",
                "/api/gi/wishes-import/{uid}",
                "/api/zzz/signals-import/{uid}",
            ] {
                assert!(!paths.contains_key(path), "legacy path remains: {path}");
            }
        }
    }

    mod banner_routes {
        #[test]
        fn openapi_keeps_hsr_and_gi_banner_routes_distinct() {
            let openapi = crate::api::openapi();
            let paths = &openapi.paths.paths;

            assert!(paths.contains_key("/api/banners/{id}"));
            assert!(paths.contains_key("/api/gi/banners/{id}"));
        }

        #[actix_web::test]
        async fn resolve_to_their_own_game() {
            use actix_web::{http::StatusCode, test, web, App};
            use sqlx::postgres::PgPoolOptions;

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

            let id = 1_500_000_000 + (uuid::Uuid::new_v4().as_u128() % 100_000_000) as i32;
            sqlx::query("INSERT INTO characters (id, rarity) VALUES ($1, 5)")
                .bind(id)
                .execute(&pool)
                .await
                .expect("HSR character fixture inserts");
            sqlx::query("INSERT INTO gi_characters (id, rarity) VALUES ($1, 5)")
                .bind(id)
                .execute(&pool)
                .await
                .expect("GI character fixture inserts");
            sqlx::query(
                "INSERT INTO banners (id, start, \"end\", character, character_gacha_type, name) VALUES ($1, now(), now() + interval '1 day', $1, 11, 'hsr-route')",
            )
            .bind(id)
            .execute(&pool)
            .await
            .expect("HSR banner fixture inserts");
            sqlx::query(
                "INSERT INTO gi_banners (id, start, \"end\", character, character_gacha_type, name) VALUES ($1, now(), now() + interval '1 day', $1, 301, 'gi-route')",
            )
            .bind(id)
            .execute(&pool)
            .await
            .expect("GI banner fixture inserts");

            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(pool.clone()))
                    .configure(crate::api::banners::configure)
                    .configure(crate::api::gi::configure),
            )
            .await;

            let hsr = test::call_service(
                &app,
                test::TestRequest::get()
                    .uri(&format!("/api/banners/{id}"))
                    .to_request(),
            )
            .await;
            assert_eq!(hsr.status(), StatusCode::OK);
            let hsr_body: serde_json::Value = test::read_body_json(hsr).await;
            assert_eq!(hsr_body["name"], "hsr-route");

            let gi = test::call_service(
                &app,
                test::TestRequest::get()
                    .uri(&format!("/api/gi/banners/{id}"))
                    .to_request(),
            )
            .await;
            assert_eq!(gi.status(), StatusCode::OK);
            let gi_body: serde_json::Value = test::read_body_json(gi).await;
            assert_eq!(gi_body["name"], "gi-route");

            sqlx::query("DELETE FROM banners WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .expect("HSR banner fixture cleans up");
            sqlx::query("DELETE FROM gi_banners WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .expect("GI banner fixture cleans up");
            sqlx::query("DELETE FROM characters WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .expect("HSR character fixture cleans up");
            sqlx::query("DELETE FROM gi_characters WHERE id = $1")
                .bind(id)
                .execute(&pool)
                .await
                .expect("GI character fixture cleans up");
        }
    }
}

#[derive(Debug)]
pub(crate) struct InvalidImportResponse;
impl std::fmt::Display for InvalidImportResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("invalid import response")
    }
}
impl std::error::Error for InvalidImportResponse {}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    #[actix_web::test]
    async fn completed_jobs_remain_pollable_and_new_starts_do_not_join_them() {
        let store = ImportJobStore::default();
        let first = store.start(1, ()).await;
        store.complete(first.id).await;
        store.complete(first.id).await;
        assert!(store.get(first.id).await.is_some());
        let next = store.start(1, ()).await;
        assert!(next.is_new);
        assert_ne!(first.id, next.id);
        assert_eq!(store.start(1, ()).await.id, next.id);
    }
}

#[cfg(test)]
mod retention_tests {
    use super::*;
    #[actix_web::test]
    async fn repeated_completion_does_not_extend_retention_or_remove_new_job() {
        let store = ImportJobStore::default();
        let first = store.start(1, ()).await;
        store
            .complete_with_retention(first.id, std::time::Duration::from_millis(20))
            .await;
        store
            .complete_with_retention(first.id, std::time::Duration::from_secs(60))
            .await;
        let second = store.start(1, ()).await;
        assert!(store.get(first.id).await.is_some());
        actix_web::rt::time::sleep(std::time::Duration::from_millis(40)).await;
        assert!(store.get(first.id).await.is_none());
        assert!(store.get(second.id).await.is_some());
    }
}
