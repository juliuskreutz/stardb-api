use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, Transaction};
#[derive(FromRow)]
pub struct DbCompletion {
    pub kind: String,
    pub id: String,
}

#[derive(FromRow)]
pub struct DbSetting {
    pub namespace: String,
    pub data: Value,
}

#[derive(FromRow)]
pub struct DbAchievementStats {
    pub id: String,
    pub completed_users: i64,
    pub total_users: i64,
    pub percent: f64,
}

#[derive(FromRow)]
pub struct DbMarkerComment {
    pub id: i64,
    pub marker_key: String,
    pub username: String,
    pub body: String,
    pub screenshot_urls: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub score: i64,
    pub upvotes: i64,
    pub downvotes: i64,
    pub viewer_vote: i32,
    pub owned_by_viewer: bool,
}

pub async fn get_completions(username: &str, pool: &PgPool) -> Result<Vec<DbCompletion>> {
    let user_id = get_user_id(username, pool).await?;

    Ok(sqlx::query_as!(
        DbCompletion,
        "SELECT kind, id FROM ntehelper_user_completion WHERE user_id = $1 ORDER BY kind, id",
        user_id,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn get_settings(username: &str, pool: &PgPool) -> Result<Vec<DbSetting>> {
    let user_id = get_user_id(username, pool).await?;

    Ok(sqlx::query_as!(
        DbSetting,
        r#"SELECT namespace, data as "data: Value" FROM ntehelper_user_settings WHERE user_id = $1 ORDER BY namespace"#,
        user_id,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn replace_state(
    username: &str,
    completions: &[DbCompletion],
    settings: &[DbSetting],
    pool: &PgPool,
) -> Result<()> {
    let user_id = get_user_id(username, pool).await?;
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM ntehelper_user_completion WHERE user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "DELETE FROM ntehelper_user_settings WHERE user_id = $1",
        user_id
    )
    .execute(&mut *tx)
    .await?;

    for completion in completions {
        insert_completion(user_id, &completion.kind, &completion.id, &mut tx).await?;
    }

    for setting in settings {
        upsert_setting(user_id, &setting.namespace, &setting.data, &mut tx).await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn patch_completions(
    username: &str,
    add: &[DbCompletion],
    remove: &[DbCompletion],
    pool: &PgPool,
) -> Result<()> {
    let user_id = get_user_id(username, pool).await?;
    let mut tx = pool.begin().await?;

    for completion in add {
        insert_completion(user_id, &completion.kind, &completion.id, &mut tx).await?;
    }

    for completion in remove {
        sqlx::query!(
            "DELETE FROM ntehelper_user_completion WHERE user_id = $1 AND kind = $2 AND id = $3",
            user_id,
            &completion.kind,
            &completion.id,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}

pub async fn set_setting(
    username: &str,
    namespace: &str,
    data: &Value,
    pool: &PgPool,
) -> Result<()> {
    let user_id = get_user_id(username, pool).await?;

    sqlx::query!(
        "INSERT INTO ntehelper_user_settings (user_id, namespace, data) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, namespace) DO UPDATE SET data = EXCLUDED.data, updated_at = now()",
        user_id,
        namespace,
        data,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn achievement_stats(pool: &PgPool) -> Result<Vec<DbAchievementStats>> {
    Ok(sqlx::query_as!(
        DbAchievementStats,
        r#"WITH total AS (
             SELECT COUNT(DISTINCT user_id)::bigint AS total_users
             FROM ntehelper_user_completion
             WHERE kind = 'achievement'
         )
         SELECT
             id AS "id!",
             COUNT(DISTINCT user_id)::bigint AS "completed_users!",
             total.total_users AS "total_users!",
             CASE
                 WHEN total.total_users = 0 THEN 0::float8
                 ELSE COUNT(DISTINCT user_id)::float8 / total.total_users::float8
             END AS "percent!"
         FROM ntehelper_user_completion
         CROSS JOIN total
         WHERE kind = 'achievement'
         GROUP BY id, total.total_users
         ORDER BY id"#,
    )
    .fetch_all(pool)
    .await?)
}

pub async fn list_marker_comments(
    marker_key: &str,
    viewer_username: Option<&str>,
    limit: i64,
    offset: i64,
    pool: &PgPool,
) -> Result<Vec<DbMarkerComment>> {
    let viewer_user_id = if let Some(username) = viewer_username {
        get_user_id_optional(username, pool).await?
    } else {
        None
    };

    Ok(sqlx::query_as::<_, DbMarkerComment>(
        r#"SELECT
             c.id,
             c.marker_key,
             u.username,
             c.body,
             c.screenshot_urls,
             c.created_at,
             c.updated_at,
             COALESCE(SUM(v.value), 0)::bigint AS score,
             COUNT(*) FILTER (WHERE v.value = 1)::bigint AS upvotes,
             COUNT(*) FILTER (WHERE v.value = -1)::bigint AS downvotes,
             COALESCE(viewer_vote.value, 0)::int AS viewer_vote,
             COALESCE(c.user_id = $2, false) AS owned_by_viewer
           FROM ntehelper_marker_comment c
           JOIN users u ON u.id = c.user_id
           LEFT JOIN ntehelper_marker_comment_vote v ON v.comment_id = c.id
           LEFT JOIN ntehelper_marker_comment_vote viewer_vote
             ON viewer_vote.comment_id = c.id AND viewer_vote.user_id = $2
           WHERE c.marker_key = $1 AND c.deleted_at IS NULL
           GROUP BY c.id, c.marker_key, c.user_id, u.username, c.body, c.screenshot_urls, c.created_at, c.updated_at, viewer_vote.value
           ORDER BY score DESC, c.created_at DESC, c.id DESC
           OFFSET $3
           LIMIT $4"#,
    )
    .bind(marker_key)
    .bind(viewer_user_id)
    .bind(offset)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn create_marker_comment(
    username: &str,
    marker_key: &str,
    body: &str,
    screenshot_urls: &Value,
    pool: &PgPool,
) -> Result<DbMarkerComment> {
    let user_id = get_user_id(username, pool).await?;
    let comment_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO ntehelper_marker_comment (marker_key, user_id, body, screenshot_urls) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(marker_key)
    .bind(user_id)
    .bind(body)
    .bind(screenshot_urls)
    .fetch_one(pool)
    .await?;

    Ok(get_marker_comment(comment_id, Some(user_id), pool)
        .await?
        .expect("created marker comment should be visible"))
}

pub async fn update_marker_comment(
    comment_id: i64,
    username: &str,
    body: &str,
    screenshot_urls: &Value,
    pool: &PgPool,
) -> Result<Option<DbMarkerComment>> {
    let user_id = get_user_id(username, pool).await?;
    let updated_id = sqlx::query_scalar::<_, i64>(
        "UPDATE ntehelper_marker_comment SET body = $3, screenshot_urls = $4, updated_at = now()
         WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
         RETURNING id",
    )
    .bind(comment_id)
    .bind(user_id)
    .bind(body)
    .bind(screenshot_urls)
    .fetch_optional(pool)
    .await?;

    match updated_id {
        Some(id) => get_marker_comment(id, Some(user_id), pool).await,
        None => Ok(None),
    }
}

pub async fn delete_marker_comment(comment_id: i64, username: &str, pool: &PgPool) -> Result<bool> {
    let user_id = get_user_id(username, pool).await?;
    let result = sqlx::query(
        "UPDATE ntehelper_marker_comment SET deleted_at = now(), updated_at = now()
         WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
    )
    .bind(comment_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn set_marker_comment_vote(
    comment_id: i64,
    username: &str,
    value: i32,
    pool: &PgPool,
) -> Result<Option<DbMarkerComment>> {
    let user_id = get_user_id(username, pool).await?;
    let visible = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM ntehelper_marker_comment WHERE id = $1 AND deleted_at IS NULL)",
    )
    .bind(comment_id)
    .fetch_one(pool)
    .await?;

    if !visible {
        return Ok(None);
    }

    if value == 0 {
        sqlx::query(
            "DELETE FROM ntehelper_marker_comment_vote WHERE comment_id = $1 AND user_id = $2",
        )
        .bind(comment_id)
        .bind(user_id)
        .execute(pool)
        .await?;
    } else {
        sqlx::query(
            "INSERT INTO ntehelper_marker_comment_vote (comment_id, user_id, value)
             VALUES ($1, $2, $3)
             ON CONFLICT (comment_id, user_id)
             DO UPDATE SET value = EXCLUDED.value, updated_at = now()",
        )
        .bind(comment_id)
        .bind(user_id)
        .bind(value)
        .execute(pool)
        .await?;
    }

    get_marker_comment(comment_id, Some(user_id), pool).await
}

pub async fn marker_comment_retry_after(
    username: &str,
    max_comments_per_window: i64,
    pool: &PgPool,
) -> Result<Option<i64>> {
    let user_id = get_user_id(username, pool).await?;
    let short_retry = sqlx::query_scalar::<_, f64>(
        "SELECT GREATEST(0, EXTRACT(EPOCH FROM (created_at + interval '3 seconds' - now())))::float8
         FROM ntehelper_marker_comment
         WHERE user_id = $1
         ORDER BY created_at DESC
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .unwrap_or(0.0);

    if short_retry > 0.0 {
        return Ok(Some(short_retry.ceil() as i64));
    }

    let window_retry = sqlx::query_scalar::<_, f64>(
        "SELECT CASE WHEN COUNT(*) >= $2
             THEN GREATEST(1, EXTRACT(EPOCH FROM (MIN(created_at) + interval '30 minutes' - now())))::float8
             ELSE 0::float8
         END
         FROM ntehelper_marker_comment
         WHERE user_id = $1 AND created_at > now() - interval '30 minutes'",
    )
    .bind(user_id)
    .bind(max_comments_per_window)
    .fetch_one(pool)
    .await?;

    if window_retry > 0.0 {
        Ok(Some(window_retry.ceil() as i64))
    } else {
        Ok(None)
    }
}

async fn get_marker_comment(
    comment_id: i64,
    viewer_user_id: Option<i64>,
    pool: &PgPool,
) -> Result<Option<DbMarkerComment>> {
    Ok(sqlx::query_as::<_, DbMarkerComment>(
        r#"SELECT
             c.id,
             c.marker_key,
             u.username,
             c.body,
             c.screenshot_urls,
             c.created_at,
             c.updated_at,
             COALESCE(SUM(v.value), 0)::bigint AS score,
             COUNT(*) FILTER (WHERE v.value = 1)::bigint AS upvotes,
             COUNT(*) FILTER (WHERE v.value = -1)::bigint AS downvotes,
             COALESCE(viewer_vote.value, 0)::int AS viewer_vote,
             COALESCE(c.user_id = $2, false) AS owned_by_viewer
           FROM ntehelper_marker_comment c
           JOIN users u ON u.id = c.user_id
           LEFT JOIN ntehelper_marker_comment_vote v ON v.comment_id = c.id
           LEFT JOIN ntehelper_marker_comment_vote viewer_vote
             ON viewer_vote.comment_id = c.id AND viewer_vote.user_id = $2
           WHERE c.id = $1 AND c.deleted_at IS NULL
           GROUP BY c.id, c.marker_key, c.user_id, u.username, c.body, c.screenshot_urls, c.created_at, c.updated_at, viewer_vote.value"#,
    )
    .bind(comment_id)
    .bind(viewer_user_id)
    .fetch_optional(pool)
    .await?)
}

async fn insert_completion(
    user_id: i64,
    kind: &str,
    id: &str,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO ntehelper_user_completion (user_id, kind, id) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, kind, id) DO NOTHING",
        user_id,
        kind,
        id,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn upsert_setting(
    user_id: i64,
    namespace: &str,
    data: &Value,
    tx: &mut Transaction<'_, Postgres>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO ntehelper_user_settings (user_id, namespace, data) VALUES ($1, $2, $3)
         ON CONFLICT (user_id, namespace) DO UPDATE SET data = EXCLUDED.data, updated_at = now()",
        user_id,
        namespace,
        data,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn get_user_id(username: &str, pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query_scalar!("SELECT id FROM users WHERE username = $1", username)
            .fetch_one(pool)
            .await?,
    )
}

async fn get_user_id_optional(username: &str, pool: &PgPool) -> Result<Option<i64>> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(pool)
            .await?,
    )
}
