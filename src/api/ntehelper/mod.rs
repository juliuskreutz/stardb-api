use actix_session::Session;
use actix_web::{get, patch, put, web, HttpResponse, Responder};
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
async fn get_me(session: Session) -> ApiResult<impl Responder> {
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
async fn get_state(session: Session, pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
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
    session: Session,
    state: web::Json<StateResponse>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let completions = state.completions.to_db();
    if completions.iter().any(|c| !valid_completion_id(&c.id)) || !state.settings.is_valid() {
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
    session: Session,
    patch: web::Json<CompletionPatch>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !patch.add.iter().all(CompletionOperation::is_valid)
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
    session: Session,
    namespace: web::Path<String>,
    data: web::Json<Value>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let namespace = namespace.into_inner();
    if !valid_setting_namespace(&namespace) || !data.is_object() {
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
    !id.trim().is_empty()
}

fn valid_setting_namespace(namespace: &str) -> bool {
    SETTING_NAMESPACES.contains(&namespace)
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
        self.tasks.is_object()
            && self.quests.is_object()
            && self.achievements.is_object()
            && self.map.is_object()
            && self.stamina.is_object()
            && self.global.is_object()
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
