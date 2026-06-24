mod tracker;

use actix_session::Session;
use actix_web::{delete, get, patch, post, put, web, HttpRequest, HttpResponse, Responder};
use reqwest::header;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::time::Duration;

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
const DEFAULT_MARKER_COMMENT_LIMIT: i64 = 10;
const MAX_MARKER_COMMENT_LIMIT: i64 = 50;
const MAX_MARKER_COMMENT_BODY_CHARS: usize = 250;
const MARKER_COMMENT_CREATE_LIMIT_PER_WINDOW: i64 = 60;
const MAX_MARKER_COMMENT_SCREENSHOT_URLS: usize = 4;
const MAX_MARKER_COMMENT_SCREENSHOT_URL_CHARS: usize = 2048;
const MARKER_COMMENT_SCREENSHOT_VALIDATE_TIMEOUT_SECS: u64 = 8;

#[derive(OpenApi)]
#[openapi(
    tags((name = "ntehelper")),
    paths(
        get_me,
        get_state,
        put_state,
        patch_completions,
        put_setting,
        get_achievement_stats,
        get_marker_comments,
        post_marker_comment,
        patch_marker_comment,
        delete_marker_comment,
        put_marker_comment_vote
    ),
    components(schemas(
        MeResponse,
        MarkerCommentCreateRequest,
        MarkerCommentListQuery,
        MarkerCommentListResponse,
        MarkerCommentResponse,
        MarkerCommentUpdateRequest,
        MarkerCommentVoteRequest,
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
    let mut openapi = ApiDoc::openapi();
    openapi.merge(tracker::openapi());
    openapi
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_me)
        .service(get_state)
        .service(put_state)
        .service(patch_completions)
        .service(put_setting)
        .service(get_achievement_stats)
        .service(get_marker_comments)
        .service(post_marker_comment)
        .service(patch_marker_comment)
        .service(delete_marker_comment)
        .service(put_marker_comment_vote)
        .configure(tracker::configure);
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

#[derive(Deserialize, ToSchema)]
struct MarkerCommentListQuery {
    #[serde(rename = "markerKey")]
    marker_key: String,
    limit: Option<i64>,
    cursor: Option<String>,
}

#[derive(Serialize, ToSchema)]
struct MarkerCommentListResponse {
    comments: Vec<MarkerCommentResponse>,
    #[serde(rename = "nextCursor")]
    next_cursor: Option<String>,
}

#[derive(Serialize, ToSchema)]
struct MarkerCommentResponse {
    id: String,
    #[serde(rename = "markerKey")]
    marker_key: String,
    username: String,
    body: String,
    #[serde(rename = "screenshotUrls")]
    screenshot_urls: Vec<String>,
    #[serde(rename = "createdAt")]
    created_at: chrono::DateTime<chrono::Utc>,
    #[serde(rename = "updatedAt")]
    updated_at: chrono::DateTime<chrono::Utc>,
    score: i64,
    upvotes: i64,
    downvotes: i64,
    #[serde(rename = "viewerVote")]
    viewer_vote: i32,
    #[serde(rename = "ownedByViewer")]
    owned_by_viewer: bool,
}

#[derive(Deserialize, ToSchema)]
struct MarkerCommentCreateRequest {
    #[serde(rename = "markerKey")]
    marker_key: String,
    body: String,
    #[serde(rename = "screenshotUrls", default)]
    screenshot_urls: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
struct MarkerCommentUpdateRequest {
    body: String,
    #[serde(rename = "screenshotUrls", default)]
    screenshot_urls: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
struct MarkerCommentVoteRequest {
    value: i32,
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

#[utoipa::path(
    tag = "ntehelper",
    get,
    path = "/api/ntehelper/marker-comments",
    responses((status = 200, description = "Marker comments", body = MarkerCommentListResponse))
)]
#[get("/api/ntehelper/marker-comments")]
async fn get_marker_comments(
    query: web::Query<MarkerCommentListQuery>,
    session: Session,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !valid_marker_key(&query.marker_key) {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let viewer_username = session.get::<String>("username").ok().flatten();
    let limit = query
        .limit
        .unwrap_or(DEFAULT_MARKER_COMMENT_LIMIT)
        .clamp(1, MAX_MARKER_COMMENT_LIMIT);
    let offset = query
        .cursor
        .as_deref()
        .and_then(|cursor| cursor.parse::<i64>().ok())
        .filter(|offset| *offset >= 0)
        .unwrap_or(0);

    let mut comments = database::ntehelper::list_marker_comments(
        &query.marker_key,
        viewer_username.as_deref(),
        limit + 1,
        offset,
        &pool,
    )
    .await?;
    let next_cursor = if comments.len() > limit as usize {
        comments.truncate(limit as usize);
        Some((offset + limit).to_string())
    } else {
        None
    };

    Ok(HttpResponse::Ok().json(MarkerCommentListResponse {
        comments: comments
            .into_iter()
            .map(MarkerCommentResponse::from)
            .collect(),
        next_cursor,
    }))
}

#[utoipa::path(
    tag = "ntehelper",
    post,
    path = "/api/ntehelper/marker-comments",
    request_body = MarkerCommentCreateRequest,
    responses(
        (status = 200, description = "Created marker comment", body = MarkerCommentResponse),
        (status = 400, description = "Not logged in or invalid comment"),
        (status = 429, description = "Comment rate limit reached"),
    )
)]
#[post("/api/ntehelper/marker-comments")]
async fn post_marker_comment(
    request: HttpRequest,
    session: Session,
    data: web::Json<MarkerCommentCreateRequest>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let body = data.body.trim();
    let Some(screenshot_urls) = normalize_marker_comment_screenshot_urls(&data.screenshot_urls)
    else {
        return Ok(HttpResponse::BadRequest().body("Invalid screenshot URLs"));
    };
    if !validate_marker_comment_screenshot_urls(&screenshot_urls).await {
        return Ok(HttpResponse::BadRequest().body("Invalid screenshot URLs"));
    };
    if !valid_marker_key(&data.marker_key) || !valid_marker_comment_body(body) {
        return Ok(HttpResponse::BadRequest().finish());
    }

    if let Some(retry_after) = database::ntehelper::marker_comment_retry_after(
        &username,
        MARKER_COMMENT_CREATE_LIMIT_PER_WINDOW,
        &pool,
    )
    .await?
    {
        return Ok(HttpResponse::TooManyRequests()
            .append_header(("Retry-After", retry_after.to_string()))
            .finish());
    }

    let comment = database::ntehelper::create_marker_comment(
        &username,
        &data.marker_key,
        body,
        &Value::Array(
            screenshot_urls
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
        &pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(MarkerCommentResponse::from(comment)))
}

#[utoipa::path(
    tag = "ntehelper",
    patch,
    path = "/api/ntehelper/marker-comments/{commentId}",
    request_body = MarkerCommentUpdateRequest,
    responses((status = 200, description = "Updated marker comment", body = MarkerCommentResponse))
)]
#[patch("/api/ntehelper/marker-comments/{comment_id}")]
async fn patch_marker_comment(
    request: HttpRequest,
    session: Session,
    comment_id: web::Path<i64>,
    data: web::Json<MarkerCommentUpdateRequest>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    let body = data.body.trim();
    let Some(screenshot_urls) = normalize_marker_comment_screenshot_urls(&data.screenshot_urls)
    else {
        return Ok(HttpResponse::BadRequest().body("Invalid screenshot URLs"));
    };
    if !validate_marker_comment_screenshot_urls(&screenshot_urls).await {
        return Ok(HttpResponse::BadRequest().body("Invalid screenshot URLs"));
    };
    if !valid_marker_comment_body(body) {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let Some(comment) = database::ntehelper::update_marker_comment(
        comment_id.into_inner(),
        &username,
        body,
        &Value::Array(
            screenshot_urls
                .into_iter()
                .map(Value::String)
                .collect(),
        ),
        &pool,
    )
    .await?
    else {
        return Ok(HttpResponse::Forbidden().finish());
    };

    Ok(HttpResponse::Ok().json(MarkerCommentResponse::from(comment)))
}

#[utoipa::path(
    tag = "ntehelper",
    delete,
    path = "/api/ntehelper/marker-comments/{commentId}",
    responses((status = 200, description = "Deleted marker comment"))
)]
#[delete("/api/ntehelper/marker-comments/{comment_id}")]
async fn delete_marker_comment(
    request: HttpRequest,
    session: Session,
    comment_id: web::Path<i64>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !database::ntehelper::delete_marker_comment(comment_id.into_inner(), &username, &pool)
        .await?
    {
        return Ok(HttpResponse::Forbidden().finish());
    }

    Ok(HttpResponse::Ok().json(json!({ "deleted": true })))
}

#[utoipa::path(
    tag = "ntehelper",
    put,
    path = "/api/ntehelper/marker-comments/{commentId}/vote",
    request_body = MarkerCommentVoteRequest,
    responses((status = 200, description = "Updated marker comment vote", body = MarkerCommentResponse))
)]
#[put("/api/ntehelper/marker-comments/{comment_id}/vote")]
async fn put_marker_comment_vote(
    request: HttpRequest,
    session: Session,
    comment_id: web::Path<i64>,
    data: web::Json<MarkerCommentVoteRequest>,
    pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    if !valid_nte_origin(&request) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let Ok(Some(username)) = session.get::<String>("username") else {
        return Ok(HttpResponse::BadRequest().finish());
    };

    if !matches!(data.value, -1 | 0 | 1) {
        return Ok(HttpResponse::BadRequest().finish());
    }

    let Some(comment) = database::ntehelper::set_marker_comment_vote(
        comment_id.into_inner(),
        &username,
        data.value,
        &pool,
    )
    .await?
    else {
        return Ok(HttpResponse::NotFound().finish());
    };

    Ok(HttpResponse::Ok().json(MarkerCommentResponse::from(comment)))
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

fn valid_marker_key(marker_key: &str) -> bool {
    valid_completion_id(marker_key)
}

fn valid_marker_comment_body(body: &str) -> bool {
    let char_count = body.chars().count();
    char_count >= 1 && char_count <= MAX_MARKER_COMMENT_BODY_CHARS
}

fn normalize_marker_comment_screenshot_urls(urls: &[String]) -> Option<Vec<String>> {
    if urls.len() > MAX_MARKER_COMMENT_SCREENSHOT_URLS {
        return None;
    }

    let mut normalized = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for url in urls {
        let trimmed = url.trim();
        if trimmed.is_empty() || trimmed.chars().count() > MAX_MARKER_COMMENT_SCREENSHOT_URL_CHARS {
            return None;
        }

        let parsed = url::Url::parse(trimmed).ok()?;
        let extension = parsed.path().rsplit('.').next()?.to_ascii_lowercase();
        if !matches!(parsed.scheme(), "http" | "https")
            || !matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "avif")
        {
            return None;
        }

        let value = parsed.to_string();
        if !seen.insert(value.clone()) {
            continue;
        }
        normalized.push(value);
    }

    Some(normalized)
}

async fn validate_marker_comment_screenshot_urls(urls: &[String]) -> bool {
    if urls.is_empty() {
        return true;
    }

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(
            MARKER_COMMENT_SCREENSHOT_VALIDATE_TIMEOUT_SECS,
        ))
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(_) => return false,
    };

    for url in urls {
        if !validate_marker_comment_screenshot_url(&client, url).await {
            return false;
        }
    }

    true
}

async fn validate_marker_comment_screenshot_url(client: &reqwest::Client, url: &str) -> bool {
    let mut current = match url::Url::parse(url) {
        Ok(url) => url,
        Err(_) => return false,
    };

    for _ in 0..=3 {
        if !url_targets_public_host(&current) {
            return false;
        }

        let head = match client.head(current.clone()).send().await {
            Ok(response) => response,
            Err(_) => return false,
        };

        if let Some(next) = redirect_target(&current, &head) {
            current = next;
            continue;
        }
        if response_has_image_content_type(&head) {
            return true;
        }

        let get = match client
            .get(current.clone())
            .header(header::RANGE, "bytes=0-0")
            .send()
            .await
        {
            Ok(response) => response,
            Err(_) => return false,
        };

        if let Some(next) = redirect_target(&current, &get) {
            current = next;
            continue;
        }

        return response_has_image_content_type(&get);
    }

    false
}

fn redirect_target(current: &url::Url, response: &reqwest::Response) -> Option<url::Url> {
    if !response.status().is_redirection() {
        return None;
    }

    let location = response.headers().get(header::LOCATION)?.to_str().ok()?;
    current.join(location).ok()
}

fn response_has_image_content_type(response: &reqwest::Response) -> bool {
    if !response.status().is_success() {
        return false;
    }

    response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .is_some_and(|value| value.starts_with("image/"))
}

fn url_targets_public_host(url: &url::Url) -> bool {
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }

    let Some(host) = url.host_str() else {
        return false;
    };
    let normalized_host = host.trim_start_matches('[').trim_end_matches(']');
    if !valid_public_hostname(normalized_host) {
        return false;
    }

    let port = url.port_or_known_default().unwrap_or(80);
    let Ok(addresses) = (normalized_host, port).to_socket_addrs() else {
        return false;
    };

    let mut resolved_any = false;
    for address in addresses {
        resolved_any = true;
        if !is_public_ip(address.ip()) {
            return false;
        }
    }

    resolved_any
}

fn valid_public_hostname(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost")
        || host.eq_ignore_ascii_case("localhost.localdomain")
        || host.ends_with(".local")
        || host.ends_with(".internal")
    {
        return false;
    }

    true
}

fn is_public_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(ip) => {
            let [a, b, ..] = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_broadcast()
                && !ip.is_documentation()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !(a == 100 && (64..=127).contains(&b))
                && !(a == 198 && matches!(b, 18 | 19))
                && !(a >= 240)
        }
        std::net::IpAddr::V6(ip) => {
            let segments = ip.segments();
            !ip.is_loopback()
                && !ip.is_unspecified()
                && !ip.is_multicast()
                && !ip.is_unique_local()
                && !ip.is_unicast_link_local()
                && !(segments[0] == 0x2001 && segments[1] == 0x0db8)
        }
    }
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

pub(super) fn valid_nte_origin(request: &HttpRequest) -> bool {
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

impl From<database::ntehelper::DbMarkerComment> for MarkerCommentResponse {
    fn from(comment: database::ntehelper::DbMarkerComment) -> Self {
        MarkerCommentResponse {
            id: comment.id.to_string(),
            marker_key: comment.marker_key,
            username: comment.username,
            body: comment.body,
            screenshot_urls: comment
                .screenshot_urls
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect(),
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            score: comment.score,
            upvotes: comment.upvotes,
            downvotes: comment.downvotes,
            viewer_vote: comment.viewer_vote,
            owned_by_viewer: comment.owned_by_viewer,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::valid_nte_origin;
    use super::valid_nte_origin_value;
    use super::{is_public_ip, valid_public_hostname};
    use actix_web::test::TestRequest;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

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

    #[test]
    fn rejects_local_or_internal_marker_screenshot_hosts() {
        assert!(!valid_public_hostname("localhost"));
        assert!(!valid_public_hostname("printer.local"));
        assert!(!valid_public_hostname("api.internal"));
        assert!(valid_public_hostname("i.imgur.com"));
    }

    #[test]
    fn rejects_non_public_marker_screenshot_ips() {
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(169, 254, 1, 1))));
        assert!(!is_public_ip(IpAddr::V4(Ipv4Addr::new(100, 64, 0, 1))));
        assert!(!is_public_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));
        assert!(!is_public_ip(IpAddr::V6(Ipv6Addr::UNSPECIFIED)));
        assert!(!is_public_ip(IpAddr::V6(
            "fc00::1".parse().expect("valid unique-local IPv6 address"),
        )));
    }
}
