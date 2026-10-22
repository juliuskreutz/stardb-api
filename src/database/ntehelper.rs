//! Persistence for NTE completion state, preferences, and community marker comments.
//! Completion replacement is transactional; comment writes enforce ownership in SQL,
//! while request validation and posting limits belong to the API layer.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

// Keep list and single-comment reads on the same vote aggregate and viewer ownership rules.
// Callers bind $2 to the optional viewer ID and append their own key predicate/pagination.
const MARKER_COMMENT_SELECT: &str = r#"SELECT
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
             ON viewer_vote.comment_id = c.id AND viewer_vote.user_id = $2"#;

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

/// Lists a known user's completed IDs in kind/ID order; a missing user is an error.
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

/// Loads a known user's JSON settings ordered by namespace.
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

/// Atomically replaces all completion and setting rows for a known user.
///
/// Any failed insert or setting update rolls back both deletions and every preceding write.
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

    insert_completions(user_id, completions, &mut tx).await?;

    for setting in settings {
        upsert_setting(user_id, &setting.namespace, &setting.data, &mut tx).await?;
    }

    tx.commit().await?;

    Ok(())
}

/// Applies additions before removals in one transaction.
///
/// Duplicate additions are idempotent; a completion in both inputs ends up removed.
pub async fn patch_completions(
    username: &str,
    add: &[DbCompletion],
    remove: &[DbCompletion],
    pool: &PgPool,
) -> Result<()> {
    let user_id = get_user_id(username, pool).await?;
    let mut tx = pool.begin().await?;

    insert_completions(user_id, add, &mut tx).await?;

    // Delete pairs, not independent kind/ID sets; removal still wins over addition.
    let kinds: Vec<_> = remove.iter().map(|c| c.kind.as_str()).collect();
    let ids: Vec<_> = remove.iter().map(|c| c.id.as_str()).collect();
    sqlx::query(
        "DELETE FROM ntehelper_user_completion c
         USING UNNEST($2::text[], $3::text[]) AS removed(kind, id)
         WHERE c.user_id = $1 AND c.kind = removed.kind AND c.id = removed.id",
    )
    .bind(user_id)
    .bind(&kinds)
    .bind(&ids)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
}

/// Upserts one JSON namespace and refreshes its update timestamp.
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

/// Computes completion fractions among users with at least one achievement completion.
///
/// The denominator excludes users who have only other kinds of completion records.
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

/// Lists visible comments by descending score, creation time, and ID.
///
/// The optional viewer affects vote/ownership annotations only. Pagination limits and
/// marker-key validation are the caller's responsibility.
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
        // Only a fixed SQL projection is interpolated; request data is bound below.
        sqlx::AssertSqlSafe(format!("{MARKER_COMMENT_SELECT} WHERE c.marker_key = $1 AND c.deleted_at IS NULL
           GROUP BY c.id, c.marker_key, c.user_id, u.username, c.body, c.screenshot_urls, c.created_at, c.updated_at, viewer_vote.value
           ORDER BY score DESC, c.created_at DESC, c.id DESC
           OFFSET $3
           LIMIT $4")),
    )
    .bind(marker_key)
    .bind(viewer_user_id)
    .bind(offset)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

/// Inserts a comment for a known user and returns its aggregate with owner annotations.
///
/// The caller must validate content and enforce posting limits before calling.
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

/// Replaces body/screenshots only for an undeleted comment owned by this username.
///
/// Returns `None` for a missing, deleted, or differently owned comment.
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

/// Soft-deletes an owned, visible comment; returns whether a row changed.
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

/// Sets one viewer's vote and returns the updated visible-comment aggregate.
///
/// A zero value removes the vote. The caller validates nonzero values as -1 or 1;
/// a missing or deleted comment returns `None`.
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

/// Returns whole seconds to wait under the short cooldown or rolling posting limit.
///
/// This only reads posting history; callers must enforce the returned delay.
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

/// Loads one visible comment using the same vote and ownership projection as the list.
async fn get_marker_comment(
    comment_id: i64,
    viewer_user_id: Option<i64>,
    pool: &PgPool,
) -> Result<Option<DbMarkerComment>> {
    Ok(sqlx::query_as::<_, DbMarkerComment>(
        // Only a fixed SQL projection is interpolated; request data is bound below.
        sqlx::AssertSqlSafe(format!("{MARKER_COMMENT_SELECT} WHERE c.id = $1 AND c.deleted_at IS NULL
           GROUP BY c.id, c.marker_key, c.user_id, u.username, c.body, c.screenshot_urls, c.created_at, c.updated_at, viewer_vote.value")),
    )
    .bind(comment_id)
    .bind(viewer_user_id)
    .fetch_optional(pool)
    .await?)
}

/// Inserts completion pairs idempotently in one statement in the caller's transaction.
/// Both arrays come from the same slice so UNNEST cannot pad an unmatched element.
async fn insert_completions(
    user_id: i64,
    completions: &[DbCompletion],
    tx: &mut Transaction<'_, Postgres>,
) -> Result<()> {
    let kinds: Vec<_> = completions.iter().map(|c| c.kind.as_str()).collect();
    let ids: Vec<_> = completions.iter().map(|c| c.id.as_str()).collect();
    sqlx::query(
        "INSERT INTO ntehelper_user_completion (user_id, kind, id)
         SELECT $1, kind, id FROM UNNEST($2::text[], $3::text[]) AS input(kind, id)
         ON CONFLICT (user_id, kind, id) DO NOTHING",
    )
    .bind(user_id)
    .bind(&kinds)
    .bind(&ids)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Replaces a namespace's JSON within the caller's transaction.
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

/// Resolves a required username, returning a database error when it does not exist.
async fn get_user_id(username: &str, pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query_scalar!("SELECT id FROM users WHERE username = $1", username)
            .fetch_one(pool)
            .await?,
    )
}

/// Resolves an optional viewer without treating an unknown username as an error.
async fn get_user_id_optional(username: &str, pool: &PgPool) -> Result<Option<i64>> {
    Ok(
        sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(pool)
            .await?,
    )
}

#[cfg(test)]
mod marker_comment_tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    #[actix_web::test]
    async fn create_list_update_vote_and_delete_roundtrip() {
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL required"))
            .await
            .unwrap();
        sqlx::migrate!().run(&pool).await.unwrap();
        let owner = format!("comment_owner_{}", Uuid::new_v4().simple());
        let voter = format!("comment_voter_{}", Uuid::new_v4().simple());
        for username in [&owner, &voter] {
            sqlx::query("INSERT INTO users (username, password) VALUES ($1, 'test')")
                .bind(username)
                .execute(&pool)
                .await
                .unwrap();
        }
        let marker = format!("test-marker-{}", Uuid::new_v4());
        let screenshots = serde_json::json!([]);
        let created = create_marker_comment(&owner, &marker, "original", &screenshots, &pool)
            .await
            .unwrap();
        assert!(created.owned_by_viewer);
        assert_eq!(created.score, 0);
        let list = list_marker_comments(&marker, None, 10, 0, &pool)
            .await
            .unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, created.id);
        assert!(!list[0].owned_by_viewer);
        assert!(
            update_marker_comment(created.id, &voter, "forbidden", &screenshots, &pool)
                .await
                .unwrap()
                .is_none()
        );
        let updated = update_marker_comment(created.id, &owner, "edited", &screenshots, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.body, "edited");
        let voted = set_marker_comment_vote(created.id, &voter, 1, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            (
                voted.score,
                voted.upvotes,
                voted.downvotes,
                voted.viewer_vote
            ),
            (1, 1, 0, 1)
        );
        let list = list_marker_comments(&marker, Some(&owner), 10, 0, &pool)
            .await
            .unwrap();
        assert_eq!(
            (list[0].score, list[0].viewer_vote, list[0].owned_by_viewer),
            (1, 0, true)
        );
        let changed = set_marker_comment_vote(created.id, &voter, -1, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            (
                changed.score,
                changed.upvotes,
                changed.downvotes,
                changed.viewer_vote
            ),
            (-1, 0, 1, -1)
        );
        let cleared = set_marker_comment_vote(created.id, &voter, 0, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(cleared.score, 0);
        assert!(!delete_marker_comment(created.id, &voter, &pool)
            .await
            .unwrap());
        assert!(delete_marker_comment(created.id, &owner, &pool)
            .await
            .unwrap());
        assert!(list_marker_comments(&marker, None, 10, 0, &pool)
            .await
            .unwrap()
            .is_empty());
        assert!(set_marker_comment_vote(created.id, &voter, 1, &pool)
            .await
            .unwrap()
            .is_none());
        sqlx::query("DELETE FROM users WHERE username = ANY($1)")
            .bind(vec![owner, voter])
            .execute(&pool)
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod completion_batch_tests {
    use super::*;

    #[sqlx::test]
    async fn sql_performance_completion_batches_preserve_pairs_and_rollback(pool: PgPool) {
        for name in ["batch-owner", "batch-other"] {
            sqlx::query("INSERT INTO users(username, password) VALUES ($1, 'test')")
                .bind(name)
                .execute(&pool)
                .await
                .unwrap();
        }
        // Exercise the maximum replacement shape, not just a singleton bulk call.
        let completions: Vec<_> = ["task", "quest", "achievement", "marker"]
            .into_iter()
            .flat_map(|kind| {
                (0..5_000).map(move |id| DbCompletion {
                    kind: kind.into(),
                    id: id.to_string(),
                })
            })
            .collect();
        let settings = [DbSetting {
            namespace: "map".into(),
            data: serde_json::json!({"zoom": 3}),
        }];
        replace_state("batch-owner", &completions, &settings, &pool)
            .await
            .unwrap();
        replace_state("batch-other", &completions[..2], &[], &pool)
            .await
            .unwrap();
        assert_eq!(
            get_completions("batch-owner", &pool).await.unwrap().len(),
            20_000
        );
        let pair = |kind: &str, id: &str| DbCompletion {
            kind: kind.into(),
            id: id.into(),
        };
        // Removal must match tuples: task/1 and quest/0 must survive.
        patch_completions(
            "batch-owner",
            &[pair("task", "0"), pair("task", "0")],
            &[pair("task", "0"), pair("quest", "1")],
            &pool,
        )
        .await
        .unwrap();
        let remaining = get_completions("batch-owner", &pool).await.unwrap();
        assert_eq!(remaining.len(), 19_998);
        assert!(remaining.iter().any(|c| c.kind == "task" && c.id == "1"));
        assert!(remaining.iter().any(|c| c.kind == "quest" && c.id == "0"));
        assert_eq!(
            get_completions("batch-other", &pool).await.unwrap().len(),
            2
        );
        // A failure after replacement deletes must restore both completions and settings.
        assert!(
            replace_state("batch-owner", &[pair("invalid", "0")], &[], &pool)
                .await
                .is_err()
        );
        assert_eq!(
            get_completions("batch-owner", &pool).await.unwrap().len(),
            19_998
        );
        assert_eq!(
            get_settings("batch-owner", &pool).await.unwrap()[0].data,
            settings[0].data
        );
        let bad_settings = [DbSetting {
            namespace: "invalid".into(),
            data: serde_json::json!({}),
        }];
        assert!(replace_state("batch-owner", &[], &bad_settings, &pool)
            .await
            .is_err());
        assert_eq!(
            get_completions("batch-owner", &pool).await.unwrap().len(),
            19_998
        );
        patch_completions("batch-owner", &[], &[], &pool)
            .await
            .unwrap();
        replace_state("batch-owner", &[], &[], &pool).await.unwrap();
        assert!(get_completions("batch-owner", &pool)
            .await
            .unwrap()
            .is_empty());
        assert!(get_settings("batch-owner", &pool).await.unwrap().is_empty());
    }
}
