use std::{collections::HashMap, sync::Arc};

use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub(crate) struct ImportJobId(Uuid);

impl ImportJobId {
    pub(crate) fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for ImportJobId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Clone, Copy, Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImportErrorCode {
    UpstreamUnavailable,
    InvalidResponse,
    PersistenceFailed,
    CalculationFailed,
}

pub(crate) fn classify_import_error(
    error: &(dyn std::error::Error + 'static),
    fallback: ImportErrorCode,
) -> ImportErrorCode {
    match error.downcast_ref::<reqwest::Error>() {
        Some(error) if error.is_decode() => ImportErrorCode::InvalidResponse,
        Some(_) => ImportErrorCode::UpstreamUnavailable,
        None => fallback,
    }
}

pub(crate) fn redacted_error(error: Box<dyn std::error::Error>) -> String {
    match error.downcast::<reqwest::Error>() {
        Ok(error) => error.without_url().to_string(),
        Err(error) => error.to_string(),
    }
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImportStatus {
    Pending,
    Calculating,
    Finished,
    Error(ImportErrorCode),
}

struct ImportJob<T> {
    uid: Option<i32>,
    info: Arc<Mutex<T>>,
}

struct ImportJobs<T> {
    jobs_by_id: HashMap<ImportJobId, ImportJob<T>>,
    active_job_by_uid: HashMap<i32, ImportJobId>,
}

impl<T> Default for ImportJobs<T> {
    fn default() -> Self {
        Self {
            jobs_by_id: HashMap::new(),
            active_job_by_uid: HashMap::new(),
        }
    }
}

pub(crate) struct StartedImportJob<T> {
    pub(crate) id: ImportJobId,
    pub(crate) info: Arc<Mutex<T>>,
    pub(crate) is_new: bool,
}

pub(crate) struct ImportJobStore<T> {
    jobs: Mutex<ImportJobs<T>>,
}

impl<T> Default for ImportJobStore<T> {
    fn default() -> Self {
        Self {
            jobs: Mutex::new(ImportJobs::default()),
        }
    }
}

impl<T> ImportJobStore<T> {
    pub(crate) async fn start(&self, uid: i32, initial: T) -> StartedImportJob<T> {
        let mut jobs = self.jobs.lock().await;

        if let Some(id) = jobs.active_job_by_uid.get(&uid).copied() {
            if let Some(job) = jobs.jobs_by_id.get(&id) {
                return StartedImportJob {
                    id,
                    info: job.info.clone(),
                    is_new: false,
                };
            }

            jobs.active_job_by_uid.remove(&uid);
        }

        let id = ImportJobId::new();
        let info = Arc::new(Mutex::new(initial));
        jobs.active_job_by_uid.insert(uid, id);
        jobs.jobs_by_id.insert(
            id,
            ImportJob {
                uid: Some(uid),
                info: info.clone(),
            },
        );

        StartedImportJob {
            id,
            info,
            is_new: true,
        }
    }

    pub(crate) async fn create(&self, initial: T) -> StartedImportJob<T> {
        let mut jobs = self.jobs.lock().await;
        let id = ImportJobId::new();
        let info = Arc::new(Mutex::new(initial));
        jobs.jobs_by_id.insert(
            id,
            ImportJob {
                uid: None,
                info: info.clone(),
            },
        );

        StartedImportJob {
            id,
            info,
            is_new: true,
        }
    }

    pub(crate) async fn get(&self, id: ImportJobId) -> Option<Arc<Mutex<T>>> {
        self.jobs
            .lock()
            .await
            .jobs_by_id
            .get(&id)
            .map(|job| job.info.clone())
    }

    pub(crate) async fn remove(&self, id: ImportJobId) {
        let mut jobs = self.jobs.lock().await;
        let Some(job) = jobs.jobs_by_id.remove(&id) else {
            return;
        };

        if let Some(uid) = job.uid {
            if jobs.active_job_by_uid.get(&uid) == Some(&id) {
                jobs.active_job_by_uid.remove(&uid);
            }
        }
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
        async fn unknown_job_is_not_visible() {
            let store = ImportJobStore::<()>::default();

            assert!(store.get(ImportJobId::new()).await.is_none());
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
            sqlx::query(
                "INSERT INTO banners (id, start, \"end\", character, light_cone, name) VALUES ($1, now(), now() + interval '1 day', NULL, NULL, 'hsr-route')",
            )
            .bind(id)
            .execute(&pool)
            .await
            .expect("HSR banner fixture inserts");
            sqlx::query(
                "INSERT INTO gi_banners (id, start, \"end\", character, weapon, name) VALUES ($1, now(), now() + interval '1 day', NULL, NULL, 'gi-route')",
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
        }
    }
}
