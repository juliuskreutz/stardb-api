use actix_session::Session;
use actix_web::{get, patch, put, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use serde_json::{json, Value};
use sqlx::PgPool;
use utoipa::{OpenApi, ToSchema};

use crate::{api::ApiResult, database};

const COMPLETION_KINDS: [&str; 4] = ["task", "quest", "achievement", "marker"];
const SETTING_NAMESPACES: [&str; 6] = [
    "tasks",
    "quests",
    "achievements",
    "map",
    "stamina",
    "global",
];
const MAX_COMPLETION_ID_LEN: usize = 256;
const MAX_COMPLETIONS_PER_KIND: usize = 5_000;
const MAX_PATCH_OPERATIONS: usize = 5_000;
const MAX_SETTING_BYTES: usize = 256 * 1024;
const PUBLIC_NTE_ORIGIN: &str = "https://nte.stardb.gg";

#[derive(OpenApi)]
#[openapi(
    tags((name = "ntehelper")),
    paths(get_me, get_state, put_state, patch_completions, put_setting, get_achievement_stats),
    components(schemas(
        MeResponse,
        StateResponse,
        StateCompletions,
        StateSettings,
        CompletionOperation,
        CompletionPatch,
        AchievementStatsResponse,
        AchievementStats,
    ))
)]
struct ApiDoc;

pub fn openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_me)
        .service(get_state)
        .service(put_state)
        .service(patch_completions)
        .service(put_setting)
        .service(get_achievement_stats);
}

#[derive(Serialize, ToSchema)]
struct MeResponse {
    authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct StateResponse {
    completions: StateCompletions,
    settings: StateSettings,
}

#[derive(Default, Serialize, Deserialize, ToSchema)]
struct StateCompletions {
    task: Vec<String>,
    quest: Vec<String>,
    achievement: Vec<String>,
    marker: Vec<String>,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct StateSettings {
    #[schema(value_type = Object)]
    tasks: Value,
    #[schema(value_type = Object)]
    quests: Value,
    #[schema(value_type = Object)]
    achievements: Value,
    #[schema(value_type = Object)]
    map: Value,
    #[schema(value_type = Object)]
    stamina: Value,
    #[schema(value_type = Object)]
    global: Value,
}

#[derive(Deserialize, ToSchema)]
struct CompletionOperation {
    kind: String,
    id: String,
}

#[derive(Deserialize, ToSchema)]
struct CompletionPatch {
    #[serde(default)]
    add: Vec<CompletionOperation>,
    #[serde(default)]
    remove: Vec<CompletionOperation>,
}

#[derive(Serialize, ToSchema)]
struct AchievementStats {
    completed: i64,
    #[serde(rename = "eligibleUsers")]
    eligible_users: i64,
    percent: f64,
}

#[derive(Serialize, ToSchema)]
struct AchievementStatsResponse {
    achievements: HashMap<String, AchievementStats>,
}

#[utoipa::path(
    tag = "ntehelper",
    get,
    path = "/api/ntehelper/me",
    responses((status = 200, description = "Current NTE Helper auth state", body = MeResponse))
)]
#[get("/api/ntehelper/me")]
async fn get_me(request: HttpRequest, session: Session) -> ApiResult<impl Responder> {
    if !valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let username = session.get::<String>("username").ok().flatten();

    Ok(HttpResponse::Ok().json(MeResponse {
        authenticated: username.is_some(),
        username,
    }))
}

#[utoipa::path(
    tag = "ntehelper",
    get,
    path = "/api/ntehelper/state",
    responses(
        (status = 200, description = "NTE Helper state", body = StateResponse),
        (status = 400, description = "Not logged in"),
    )
)]
#[get("/api/ntehelper/state")]
async fn get_state(
    request: HttpRequest,
    session: Session,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    Ok(HttpResponse::Ok().json(load_state(&username, &pool).await?))
}

#[utoipa::path(
    tag = "ntehelper",
    put,
    path = "/api/ntehelper/state",
    request_body = StateResponse,
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in or invalid state"),
    )
)]
#[put("/api/ntehelper/state")]
async fn put_state(
    request: HttpRequest,
    session: Session,
    state: web::Json<StateResponse>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let completions = state.completions.to_db();
    if !state.completions.is_valid()
        || completions.iter().any(|c| !valid_completion_id(&c.id))
        || !state.settings.is_valid()
    {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let settings = state.settings.to_db();

    database::ntehelper::replace_state(&username, &completions, &settings, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "ntehelper",
    patch,
    path = "/api/ntehelper/completions",
    request_body = CompletionPatch,
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in or invalid completion kind"),
    )
)]
#[patch("/api/ntehelper/completions")]
async fn patch_completions(
    request: HttpRequest,
    session: Session,
    patch: web::Json<CompletionPatch>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if patch.add.len() + patch.remove.len() > MAX_PATCH_OPERATIONS
        || !patch.add.iter().all(CompletionOperation::is_valid)
        || !patch.remove.iter().all(CompletionOperation::is_valid)
    {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let add = patch
        .add
        .iter()
        .map(CompletionOperation::to_db)
        .collect::<Vec<_>>();
    let remove = patch
        .remove
        .iter()
        .map(CompletionOperation::to_db)
        .collect::<Vec<_>>();

    database::ntehelper::patch_completions(&username, &add, &remove, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "ntehelper",
    put,
    path = "/api/ntehelper/settings/{namespace}",
    responses(
        (status = 200, description = "Success"),
        (status = 400, description = "Not logged in or invalid settings namespace"),
    )
)]
#[put("/api/ntehelper/settings/{namespace}")]
async fn put_setting(
    request: HttpRequest,
    session: Session,
    namespace: web::Path<String>,
    data: web::Json<Value>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let namespace = namespace.into_inner();
    if !valid_setting_namespace(&namespace) || !valid_setting_value(&data) {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let data = data.into_inner();
    database::ntehelper::set_setting(&username, &namespace, &data, &pool).await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    tag = "ntehelper",
    get,
    path = "/api/ntehelper/stats/achievements",
    responses((status = 200, description = "NTE Helper achievement stats", body = AchievementStatsResponse))
)]
#[get("/api/ntehelper/stats/achievements")]
async fn get_achievement_stats(pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let stats = database::ntehelper::achievement_stats(&pool)
        .await?
        .into_iter()
        .map(|stats| {
            (
                stats.id,
                AchievementStats {
                    completed: stats.completed_users,
                    eligible_users: stats.total_users,
                    percent: stats.percent,
                },
            )
        })
        .collect::<HashMap<_, _>>();

    Ok(HttpResponse::Ok().json(AchievementStatsResponse {
        achievements: stats,
    }))
}

async fn load_state(username: &str, pool: &PgPool) -> ApiResult<StateResponse> {
    let completions =
        StateCompletions::from_db(database::ntehelper::get_completions(username, pool).await?);
    let settings = StateSettings::from_db(database::ntehelper::get_settings(username, pool).await?);

    Ok(StateResponse {
        completions,
        settings,
    })
}

fn valid_completion_kind(kind: &str) -> bool {
    COMPLETION_KINDS.contains(&kind)
}

fn valid_completion_id(id: &str) -> bool {
    let id = id.trim();
    !id.is_empty()
        && id.len() <= MAX_COMPLETION_ID_LEN
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | ':' | '.' | '/'))
}

fn valid_setting_namespace(namespace: &str) -> bool {
    SETTING_NAMESPACES.contains(&namespace)
}

fn valid_setting_value(data: &Value) -> bool {
    data.is_object()
        && serde_json::to_vec(data)
            .map(|data| data.len() <= MAX_SETTING_BYTES)
            .unwrap_or(false)
}

fn valid_nte_origin(request: &HttpRequest) -> bool {
    let Some(origin) = request.headers().get("Origin") else {
        return true;
    };

    let Ok(origin) = origin.to_str() else {
        return false;
    };

    valid_nte_origin_value(origin)
}

fn valid_nte_origin_value(origin: &str) -> bool {
    if origin == PUBLIC_NTE_ORIGIN {
        return true;
    }

    let Ok(url) = url::Url::parse(origin) else {
        return false;
    };

    if url.scheme() != "http" {
        return false;
    }

    let Some(host) = url.host_str() else {
        return false;
    };

    let normalized_host = host.trim_start_matches('[').trim_end_matches(']');

    normalized_host.eq_ignore_ascii_case("localhost")
        || normalized_host
            .parse::<std::net::IpAddr>()
            .map(|address| address.is_loopback())
            .unwrap_or(false)
}

impl CompletionOperation {
    fn is_valid(&self) -> bool {
        valid_completion_kind(&self.kind) && valid_completion_id(&self.id)
    }

    fn to_db(&self) -> database::ntehelper::DbCompletion {
        database::ntehelper::DbCompletion {
            kind: self.kind.clone(),
            id: self.id.clone(),
        }
    }
}

impl StateCompletions {
    fn from_db(completions: Vec<database::ntehelper::DbCompletion>) -> Self {
        let mut state = StateCompletions::default();

        for completion in completions {
            match completion.kind.as_str() {
                "task" => state.task.push(completion.id),
                "quest" => state.quest.push(completion.id),
                "achievement" => state.achievement.push(completion.id),
                "marker" => state.marker.push(completion.id),
                _ => {}
            }
        }

        state
    }

    fn to_db(&self) -> Vec<database::ntehelper::DbCompletion> {
        let mut completions = Vec::new();

        for (kind, ids) in [
            ("task", &self.task),
            ("quest", &self.quest),
            ("achievement", &self.achievement),
            ("marker", &self.marker),
        ] {
            completions.extend(ids.iter().map(|id| database::ntehelper::DbCompletion {
                kind: kind.to_string(),
                id: id.clone(),
            }));
        }

        completions
    }

    fn is_valid(&self) -> bool {
        self.task.len() <= MAX_COMPLETIONS_PER_KIND
            && self.quest.len() <= MAX_COMPLETIONS_PER_KIND
            && self.achievement.len() <= MAX_COMPLETIONS_PER_KIND
            && self.marker.len() <= MAX_COMPLETIONS_PER_KIND
    }
}

impl StateSettings {
    fn empty() -> Self {
        StateSettings {
            tasks: json!({}),
            quests: json!({}),
            achievements: json!({}),
            map: json!({}),
            stamina: json!({}),
            global: json!({}),
        }
    }

    fn from_db(settings: Vec<database::ntehelper::DbSetting>) -> Self {
        let mut state = StateSettings::empty();

        for setting in settings {
            match setting.namespace.as_str() {
                "tasks" => state.tasks = setting.data,
                "quests" => state.quests = setting.data,
                "achievements" => state.achievements = setting.data,
                "map" => state.map = setting.data,
                "stamina" => state.stamina = setting.data,
                "global" => state.global = setting.data,
                _ => {}
            }
        }

        state
    }

    fn is_valid(&self) -> bool {
        valid_setting_value(&self.tasks)
            && valid_setting_value(&self.quests)
            && valid_setting_value(&self.achievements)
            && valid_setting_value(&self.map)
            && valid_setting_value(&self.stamina)
            && valid_setting_value(&self.global)
    }

    fn to_db(&self) -> Vec<database::ntehelper::DbSetting> {
        vec![
            database::ntehelper::DbSetting {
                namespace: "tasks".to_string(),
                data: self.tasks.clone(),
            },
            database::ntehelper::DbSetting {
                namespace: "quests".to_string(),
                data: self.quests.clone(),
            },
            database::ntehelper::DbSetting {
                namespace: "achievements".to_string(),
                data: self.achievements.clone(),
            },
            database::ntehelper::DbSetting {
                namespace: "map".to_string(),
                data: self.map.clone(),
            },
            database::ntehelper::DbSetting {
                namespace: "stamina".to_string(),
                data: self.stamina.clone(),
            },
            database::ntehelper::DbSetting {
                namespace: "global".to_string(),
                data: self.global.clone(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::valid_nte_origin;
    use super::valid_nte_origin_value;
    use actix_web::test::TestRequest;

    #[test]
    fn accepts_public_nte_origin() {
        assert!(valid_nte_origin_value("https://nte.stardb.gg"));
    }

    #[test]
    fn accepts_missing_origin_header() {
        let request = TestRequest::default().to_http_request();
        assert!(valid_nte_origin(&request));
    }

    #[test]
    fn accepts_http_loopback_origins() {
        assert!(valid_nte_origin_value("http://localhost:5173"));
        assert!(valid_nte_origin_value("http://127.0.0.1:4173"));
        assert!(valid_nte_origin_value("http://[::1]:4173"));
    }

    #[test]
    fn rejects_https_loopback_origins() {
        assert!(!valid_nte_origin_value("https://localhost"));
        assert!(!valid_nte_origin_value("https://127.0.0.1"));
        assert!(!valid_nte_origin_value("https://[::1]"));
    }

    #[test]
    fn rejects_malformed_or_foreign_origins() {
        assert!(!valid_nte_origin_value("not a url"));
        assert!(!valid_nte_origin_value("https://stardb.gg"));
        assert!(!valid_nte_origin_value("https://example.com"));
        assert!(!valid_nte_origin_value("ftp://localhost"));
    }
}
