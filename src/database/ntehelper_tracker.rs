//! NTE UID claims and raw pull persistence.
//! A detached claim retains its profile with a nullable owner. API handlers authorize
//! profile access; mutation helpers document any additional caller preconditions.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use std::{error::Error, fmt};

#[derive(Debug, FromRow, Clone)]
pub struct DbTrackerUidClaim {
    pub uid: i64,
    pub nickname: String,
    pub region: String,
    pub owner_user_id: Option<i64>,
    pub owner_username: Option<String>,
    pub claim_source: String,
    pub has_profile: bool,
    pub claimed_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, FromRow, Clone)]
pub struct DbTrackerPull {
    pub uid: i64,
    pub record_uid: String,
    pub pool_group_id: String,
    pub timestamp_raw: String,
    pub timestamp_group_ordinal: Option<i32>,
    pub roll_result: Option<i32>,
    pub result_type: Option<String>,
    pub reward_id: String,
    pub quantity: Option<i32>,
    pub imported_at: DateTime<Utc>,
}

pub struct TrackerImportResult {
    pub inserted: i64,
    pub updated: i64,
    pub total: i64,
}

#[derive(Debug, FromRow, Clone)]
struct ExistingTrackerPullRecordUid {
    pub record_uid: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TrackerClaimError {
    AlreadyClaimed,
    SelfClaimLimit,
    NotOwnedByUser,
}

impl fmt::Display for TrackerClaimError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackerClaimError::AlreadyClaimed => f.write_str("tracker UID is already claimed"),
            TrackerClaimError::SelfClaimLimit => {
                f.write_str("account already has the maximum number of self-claimed tracker UIDs")
            }
            TrackerClaimError::NotOwnedByUser => {
                f.write_str("tracker UID is not attached to this account")
            }
        }
    }
}

impl Error for TrackerClaimError {}

/// Resolves the numeric owner ID for an existing authenticated username.
pub async fn get_tracker_user_id(username: &str, pool: &PgPool) -> Result<i64> {
    Ok(
        sqlx::query_scalar!("SELECT id FROM users WHERE username = $1", username)
            .fetch_one(pool)
            .await?,
    )
}

/// Lists attached claims newest-first, including whether each has stored pulls.
pub async fn tracker_claims_for_user(
    owner_user_id: i64,
    pool: &PgPool,
) -> Result<Vec<DbTrackerUidClaim>> {
    Ok(sqlx::query_as::<_, DbTrackerUidClaim>(
        r#"SELECT
             c.uid,
             c.nickname,
             c.region,
             c.owner_user_id,
             u.username AS owner_username,
             c.claim_source,
             EXISTS(SELECT 1 FROM ntehelper_tracker_pull p WHERE p.uid = c.uid) AS has_profile,
             c.claimed_at,
             c.updated_at
           FROM ntehelper_tracker_uid_claim c
           JOIN users u ON u.id = c.owner_user_id
           WHERE c.owner_user_id = $1
           ORDER BY c.claimed_at DESC, c.uid"#,
    )
    .bind(owner_user_id)
    .fetch_all(pool)
    .await?)
}

/// Loads an attached or detached claim; only a missing UID returns `None`.
pub async fn get_tracker_claim(uid: i64, pool: &PgPool) -> Result<Option<DbTrackerUidClaim>> {
    Ok(sqlx::query_as::<_, DbTrackerUidClaim>(
        r#"SELECT
             c.uid,
             c.nickname,
             c.region,
             c.owner_user_id,
             u.username AS owner_username,
             c.claim_source,
             EXISTS(SELECT 1 FROM ntehelper_tracker_pull p WHERE p.uid = c.uid) AS has_profile,
             c.claimed_at,
             c.updated_at
           FROM ntehelper_tracker_uid_claim c
           LEFT JOIN users u ON u.id = c.owner_user_id
           WHERE c.uid = $1"#,
    )
    .bind(uid)
    .fetch_optional(pool)
    .await?)
}

/// Creates or reattaches a self-claim while enforcing the per-user claim cap.
///
/// A user-row lock serializes cap checks. Reclaiming one's own UID is idempotent;
/// competing owners and cap violations use the inner domain error, while database
/// failures use the outer error.
pub async fn claim_tracker_uid(
    owner_user_id: i64,
    uid: i64,
    max_self_claims: i64,
    pool: &PgPool,
) -> Result<std::result::Result<DbTrackerUidClaim, TrackerClaimError>> {
    let mut tx = pool.begin().await?;

    // Serialize self-claim evaluation per user so concurrent requests cannot
    // each observe the same pre-limit count and exceed the cap together.
    sqlx::query("SELECT id FROM users WHERE id = $1 FOR UPDATE")
        .bind(owner_user_id)
        .fetch_one(&mut *tx)
        .await?;

    let existing_claim = sqlx::query_scalar::<_, i64>(
        "SELECT owner_user_id FROM ntehelper_tracker_uid_claim WHERE uid = $1 AND owner_user_id IS NOT NULL",
    )
    .bind(uid)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(existing_owner_id) = existing_claim {
        tx.rollback().await?;
        if existing_owner_id == owner_user_id {
            return Ok(Ok(get_tracker_claim(uid, pool)
                .await?
                .expect("existing tracker claim should load")));
        }

        return Ok(Err(TrackerClaimError::AlreadyClaimed));
    }

    let existing_self_claim = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM ntehelper_tracker_uid_claim WHERE owner_user_id = $1 AND claim_source = 'self'",
    )
    .bind(owner_user_id)
    .fetch_one(&mut *tx)
    .await?;

    if existing_self_claim >= max_self_claims {
        tx.rollback().await?;
        return Ok(Err(TrackerClaimError::SelfClaimLimit));
    }

    let attach_existing = sqlx::query(
        "UPDATE ntehelper_tracker_uid_claim
         SET owner_user_id = $2, claim_source = 'self', updated_at = now()
         WHERE uid = $1 AND owner_user_id IS NULL",
    )
    .bind(uid)
    .bind(owner_user_id)
    .execute(&mut *tx)
    .await?;

    if attach_existing.rows_affected() == 0 {
        let insert_result = sqlx::query(
            "INSERT INTO ntehelper_tracker_uid_claim (uid, owner_user_id, claim_source, nickname) VALUES ($1, $2, 'self', '')",
        )
        .bind(uid)
        .bind(owner_user_id)
        .execute(&mut *tx)
        .await;

        if let Err(error) = insert_result {
            tx.rollback().await?;
            if let sqlx::Error::Database(database_error) = &error {
                if database_error.code().as_deref() == Some("23505") {
                    if let Some(existing) = get_tracker_claim(uid, pool).await? {
                        return Ok(if existing.owner_user_id == Some(owner_user_id) {
                            Ok(existing)
                        } else {
                            Err(TrackerClaimError::AlreadyClaimed)
                        });
                    }

                    return Ok(Err(TrackerClaimError::SelfClaimLimit));
                }
            }

            return Err(error.into());
        }
    }

    tx.commit().await?;

    Ok(Ok(get_tracker_claim(uid, pool)
        .await?
        .expect("created tracker claim should load")))
}

/// Updates only supplied nickname/region fields on a claim owned by the caller.
///
/// The inner error distinguishes a non-owned or missing row from database failures.
pub async fn update_tracker_claim(
    owner_user_id: i64,
    uid: i64,
    nickname: Option<&str>,
    region: Option<&str>,
    pool: &PgPool,
) -> Result<std::result::Result<DbTrackerUidClaim, TrackerClaimError>> {
    let result = sqlx::query(
        "UPDATE ntehelper_tracker_uid_claim
         SET nickname = COALESCE($3, nickname),
             region = COALESCE($4, region),
             updated_at = now()
         WHERE uid = $1 AND owner_user_id = $2",
    )
    .bind(uid)
    .bind(owner_user_id)
    .bind(nickname)
    .bind(region)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Ok(Err(TrackerClaimError::NotOwnedByUser));
    }

    Ok(Ok(get_tracker_claim(uid, pool)
        .await?
        .expect("updated tracker claim should load")))
}

/// Deletes an owned claim and its cascading pull history in one transaction.
///
/// Returns the pre-deletion pull count, or an inner ownership error without deleting.
pub async fn delete_tracker_uid_profile(
    owner_user_id: i64,
    uid: i64,
    pool: &PgPool,
) -> Result<std::result::Result<i64, TrackerClaimError>> {
    let mut tx = pool.begin().await?;
    let deleted_pulls = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM ntehelper_tracker_pull WHERE uid = $1",
    )
    .bind(uid)
    .fetch_one(&mut *tx)
    .await?;

    let deleted_claim = sqlx::query(
        "DELETE FROM ntehelper_tracker_uid_claim WHERE uid = $1 AND owner_user_id = $2",
    )
    .bind(uid)
    .bind(owner_user_id)
    .execute(&mut *tx)
    .await?;

    if deleted_claim.rows_affected() == 0 {
        tx.rollback().await?;
        return Ok(Err(TrackerClaimError::NotOwnedByUser));
    }

    tx.commit().await?;

    Ok(Ok(deleted_pulls))
}

/// Lists raw pulls newest-first, breaking timestamp ties by ordinal and record UID.
///
/// The caller decides whether this profile is visible; this query does not authorize.
pub async fn tracker_pulls_for_uid(uid: i64, pool: &PgPool) -> Result<Vec<DbTrackerPull>> {
    Ok(sqlx::query_as::<_, DbTrackerPull>(
        r#"SELECT
             uid,
             record_uid,
             pool_group_id,
             timestamp_raw,
             timestamp_group_ordinal,
             roll_result,
             result_type,
             reward_id,
             quantity,
             imported_at
           FROM ntehelper_tracker_pull
           WHERE uid = $1
           ORDER BY timestamp_raw DESC, timestamp_group_ordinal DESC NULLS LAST, record_uid DESC"#,
    )
    .bind(uid)
    .fetch_all(pool)
    .await?)
}

/// Counts all stored pull records for a UID, including detached profiles.
pub async fn tracker_pull_count(uid: i64, pool: &PgPool) -> Result<i64> {
    Ok(sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM ntehelper_tracker_pull WHERE uid = $1",
    )
    .bind(uid)
    .fetch_one(pool)
    .await?)
}

/// Inserts new pulls and updates mutable outcome details for existing record IDs.
///
/// The caller must authorize the UID, deduplicate record IDs, and ensure every pull
/// has the requested UID. The claim row is locked so imports serialize; exceeding
/// `max_total` returns `None` without writes. Otherwise all writes commit together.
pub async fn import_tracker_pulls(
    uid: i64,
    pulls: &[DbTrackerPull],
    max_total: i64,
    pool: &PgPool,
) -> Result<Option<TrackerImportResult>> {
    if pulls.is_empty() {
        let total = tracker_pull_count(uid, pool).await?;
        return Ok(Some(TrackerImportResult {
            inserted: 0,
            updated: 0,
            total,
        }));
    }

    let record_uids = pulls
        .iter()
        .map(|pull| pull.record_uid.clone())
        .collect::<Vec<_>>();
    let mut tx = pool.begin().await?;

    sqlx::query("SELECT uid FROM ntehelper_tracker_uid_claim WHERE uid = $1 FOR UPDATE")
        .bind(uid)
        .fetch_one(&mut *tx)
        .await?;

    let current_total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::bigint FROM ntehelper_tracker_pull WHERE uid = $1",
    )
    .bind(uid)
    .fetch_one(&mut *tx)
    .await?;
    let existing_pulls = sqlx::query_as::<_, ExistingTrackerPullRecordUid>(
        r#"SELECT
             record_uid
           FROM ntehelper_tracker_pull
           WHERE uid = $1 AND record_uid = ANY($2::text[])"#,
    )
    .bind(uid)
    .bind(&record_uids)
    .fetch_all(&mut *tx)
    .await?;
    // The claim-row lock keeps imports for this UID serialized. The API deduplicates
    // incoming IDs, so this difference counts new rows without a second anti-join.
    let new_count = i64::try_from(pulls.len() - existing_pulls.len())?;
    let existing_record_uids = existing_pulls
        .into_iter()
        .map(|pull| pull.record_uid)
        .collect::<std::collections::HashSet<_>>();

    if current_total + new_count > max_total {
        tx.rollback().await?;
        return Ok(None);
    }

    // Keep classification under the claim lock. Only repair fields may change on
    // existing records; separate statements preserve accurate insert/update counts.
    let (existing, new): (Vec<_>, Vec<_>) = pulls
        .iter()
        .partition(|pull| existing_record_uids.contains(&pull.record_uid));
    let ids: Vec<_> = existing.iter().map(|p| p.record_uid.as_str()).collect();
    let results: Vec<_> = existing.iter().map(|p| p.roll_result).collect();
    let types: Vec<_> = existing.iter().map(|p| p.result_type.as_deref()).collect();
    let quantities: Vec<_> = existing.iter().map(|p| p.quantity).collect();
    let updated = sqlx::query(
        "UPDATE ntehelper_tracker_pull p
         SET roll_result = input.roll_result, result_type = input.result_type,
             quantity = input.quantity
         FROM UNNEST($2::text[], $3::int[], $4::text[], $5::int[])
             AS input(record_uid, roll_result, result_type, quantity)
         WHERE p.uid = $1 AND p.record_uid = input.record_uid
           AND (p.roll_result, p.result_type, p.quantity)
               IS DISTINCT FROM (input.roll_result, input.result_type, input.quantity)",
    )
    .bind(uid)
    .bind(&ids)
    .bind(&results)
    .bind(&types)
    .bind(&quantities)
    .execute(&mut *tx)
    .await?
    .rows_affected() as i64;

    let ids: Vec<_> = new.iter().map(|p| p.record_uid.as_str()).collect();
    let groups: Vec<_> = new.iter().map(|p| p.pool_group_id.as_str()).collect();
    let timestamps: Vec<_> = new.iter().map(|p| p.timestamp_raw.as_str()).collect();
    let ordinals: Vec<_> = new.iter().map(|p| p.timestamp_group_ordinal).collect();
    let results: Vec<_> = new.iter().map(|p| p.roll_result).collect();
    let types: Vec<_> = new.iter().map(|p| p.result_type.as_deref()).collect();
    let rewards: Vec<_> = new.iter().map(|p| p.reward_id.as_str()).collect();
    let quantities: Vec<_> = new.iter().map(|p| p.quantity).collect();
    let inserted = sqlx::query(
        "INSERT INTO ntehelper_tracker_pull (
             uid, record_uid, pool_group_id, timestamp_raw, timestamp_group_ordinal,
             roll_result, result_type, reward_id, quantity)
         SELECT $1, input.* FROM UNNEST(
             $2::text[], $3::text[], $4::text[], $5::int[],
             $6::int[], $7::text[], $8::text[], $9::int[]) AS input",
    )
    .bind(uid)
    .bind(&ids)
    .bind(&groups)
    .bind(&timestamps)
    .bind(&ordinals)
    .bind(&results)
    .bind(&types)
    .bind(&rewards)
    .bind(&quantities)
    .execute(&mut *tx)
    .await?
    .rows_affected() as i64;

    sqlx::query("UPDATE ntehelper_tracker_uid_claim SET updated_at = now() WHERE uid = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(Some(TrackerImportResult {
        inserted,
        updated,
        total: current_total + inserted,
    }))
}

/// Deletes all pulls and updates the claim timestamp in one transaction.
///
/// The claim itself remains attached. The caller must authorize ownership first.
pub async fn clear_tracker_pulls(uid: i64, pool: &PgPool) -> Result<i64> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query("DELETE FROM ntehelper_tracker_pull WHERE uid = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    sqlx::query("UPDATE ntehelper_tracker_uid_claim SET updated_at = now() WHERE uid = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(result.rows_affected() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::users::{self, DbUser};
    use sqlx::postgres::PgPoolOptions;
    use std::sync::atomic::{AtomicI64, Ordering};
    use uuid::Uuid;

    static NEXT_UID_OFFSET: AtomicI64 = AtomicI64::new(0);

    fn next_uid() -> i64 {
        let suffix = (Utc::now().timestamp_micros() % 10_000_000_000)
            + NEXT_UID_OFFSET.fetch_add(1, Ordering::Relaxed);
        210_000_000_000 + (suffix % 10_000_000_000)
    }

    async fn test_pool() -> PgPool {
        let _ = dotenv::dotenv();
        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for DB-backed tests");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("test database should connect");
        sqlx::migrate!()
            .run(&pool)
            .await
            .expect("test database migrations should run");
        pool
    }

    async fn create_test_user(pool: &PgPool) -> (String, i64) {
        let username = format!("tracker_test_{}", Uuid::new_v4().simple());
        let user = DbUser {
            username: username.clone(),
            password: "test-password-hash".to_string(),
            email: Some(format!("{username}@example.com")),
        };
        users::set(&user, pool)
            .await
            .expect("test user should insert successfully");
        let user_id = get_tracker_user_id(&username, pool)
            .await
            .expect("test user id should load");
        (username, user_id)
    }

    async fn delete_test_user(username: &str, pool: &PgPool) {
        sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(pool)
            .await
            .expect("test user should delete successfully");
    }

    async fn delete_test_claim(uid: i64, pool: &PgPool) {
        sqlx::query("DELETE FROM ntehelper_tracker_uid_claim WHERE uid = $1")
            .bind(uid)
            .execute(pool)
            .await
            .expect("test claim should delete successfully");
    }

    fn sample_pull(uid: i64, record_uid: &str) -> DbTrackerPull {
        DbTrackerPull {
            uid,
            record_uid: record_uid.to_string(),
            pool_group_id: "Lottery_LimitedCharacter".to_string(),
            timestamp_raw: "2026-06-07 19:28:21".to_string(),
            timestamp_group_ordinal: Some(0),
            roll_result: Some(1),
            result_type: Some("single".to_string()),
            reward_id: "test_reward".to_string(),
            quantity: Some(1),
            imported_at: Utc::now(),
        }
    }

    #[actix_web::test]
    async fn tracker_claim_creation_roundtrip() {
        let pool = test_pool().await;
        let (username, user_id) = create_test_user(&pool).await;
        let uid = next_uid();

        let claim = claim_tracker_uid(user_id, uid, 3, &pool)
            .await
            .expect("claim should succeed")
            .expect("claim should be created");

        assert_eq!(claim.uid, uid);
        assert_eq!(claim.owner_user_id, Some(user_id));
        assert_eq!(claim.claim_source, "self");
        assert!(!claim.has_profile);

        let claims = tracker_claims_for_user(user_id, &pool)
            .await
            .expect("claims should load");
        assert_eq!(claims.len(), 1);
        assert_eq!(claims[0].uid, uid);

        delete_test_user(&username, &pool).await;
    }

    #[actix_web::test]
    async fn allows_three_self_claims_but_blocks_a_fourth() {
        let pool = test_pool().await;
        let (username, user_id) = create_test_user(&pool).await;

        for _ in 0..3 {
            claim_tracker_uid(user_id, next_uid(), 3, &pool)
                .await
                .expect("claim should succeed")
                .expect("claim should be created");
        }

        let error = claim_tracker_uid(user_id, next_uid(), 3, &pool)
            .await
            .expect("limit check should return a domain error")
            .expect_err("fourth self-claim should be rejected");

        assert_eq!(error, TrackerClaimError::SelfClaimLimit);

        delete_test_user(&username, &pool).await;
    }

    #[actix_web::test]
    async fn reattaches_detached_claim_with_existing_profile() {
        let pool = test_pool().await;
        let (username, user_id) = create_test_user(&pool).await;
        let uid = next_uid();

        sqlx::query(
            "INSERT INTO ntehelper_tracker_uid_claim (uid, owner_user_id, claim_source, nickname)
             VALUES ($1, NULL, 'self', 'Detached UID')",
        )
        .bind(uid)
        .execute(&pool)
        .await
        .expect("detached claim should insert");
        import_tracker_pulls(uid, &[sample_pull(uid, "reattach-record")], 10, &pool)
            .await
            .expect("import should succeed")
            .expect("import should not hit the limit");

        claim_tracker_uid(user_id, uid, 3, &pool)
            .await
            .expect("reattach should succeed")
            .expect("detached claim should reattach");

        let claim = get_tracker_claim(uid, &pool)
            .await
            .expect("claim should load")
            .expect("claim should exist");
        assert_eq!(claim.owner_user_id, Some(user_id));
        assert!(claim.has_profile);
        assert_eq!(
            tracker_pull_count(uid, &pool)
                .await
                .expect("pull count should load"),
            1
        );

        delete_test_claim(uid, &pool).await;
        delete_test_user(&username, &pool).await;
    }

    #[actix_web::test]
    async fn clear_pulls_preserves_the_claim() {
        let pool = test_pool().await;
        let (username, user_id) = create_test_user(&pool).await;
        let uid = next_uid();

        claim_tracker_uid(user_id, uid, 3, &pool)
            .await
            .expect("claim should succeed")
            .expect("claim should be created");
        import_tracker_pulls(
            uid,
            &[sample_pull(uid, "clear-1"), sample_pull(uid, "clear-2")],
            10,
            &pool,
        )
        .await
        .expect("import should succeed")
        .expect("import should not hit the limit");

        let deleted = clear_tracker_pulls(uid, &pool)
            .await
            .expect("clear should succeed");

        assert_eq!(deleted, 2);
        assert_eq!(
            tracker_pull_count(uid, &pool)
                .await
                .expect("pull count should load"),
            0
        );
        let claim = get_tracker_claim(uid, &pool)
            .await
            .expect("claim should load")
            .expect("claim should still exist");
        assert_eq!(claim.owner_user_id, Some(user_id));
        assert!(!claim.has_profile);

        delete_test_user(&username, &pool).await;
    }

    #[actix_web::test]
    async fn import_limit_rejects_records_over_the_cap() {
        let pool = test_pool().await;
        let (username, user_id) = create_test_user(&pool).await;
        let uid = next_uid();

        claim_tracker_uid(user_id, uid, 3, &pool)
            .await
            .expect("claim should succeed")
            .expect("claim should be created");
        import_tracker_pulls(
            uid,
            &[sample_pull(uid, "limit-1"), sample_pull(uid, "limit-2")],
            2,
            &pool,
        )
        .await
        .expect("initial import should succeed")
        .expect("initial import should fit inside the limit");

        let result = import_tracker_pulls(uid, &[sample_pull(uid, "limit-3")], 2, &pool)
            .await
            .expect("limit check should succeed");

        assert!(result.is_none());
        assert_eq!(
            tracker_pull_count(uid, &pool)
                .await
                .expect("pull count should load"),
            2
        );

        delete_test_user(&username, &pool).await;
    }

    #[actix_web::test]
    async fn reimport_updates_only_safe_repair_fields_for_duplicate_pull() {
        let pool = test_pool().await;
        let (username, user_id) = create_test_user(&pool).await;
        let uid = next_uid();

        claim_tracker_uid(user_id, uid, 3, &pool)
            .await
            .expect("claim should succeed")
            .expect("claim should be created");

        let mut original_pull = sample_pull(uid, "repair-1");
        original_pull.reward_id = "1076".to_string();
        import_tracker_pulls(uid, std::slice::from_ref(&original_pull), 10, &pool)
            .await
            .expect("initial import should succeed")
            .expect("initial import should not hit the limit");

        let mut repaired_pull = original_pull.clone();
        repaired_pull.reward_id = "1076_changed".to_string();
        repaired_pull.timestamp_raw = "2030-01-01 00:00:00".to_string();
        repaired_pull.timestamp_group_ordinal = Some(99);
        repaired_pull.roll_result = Some(99);
        repaired_pull.result_type = Some("dice".to_string());
        repaired_pull.quantity = Some(9);

        let result = import_tracker_pulls(uid, &[repaired_pull], 10, &pool)
            .await
            .expect("repair import should succeed")
            .expect("repair import should not hit the limit");
        assert_eq!(result.inserted, 0);
        assert_eq!(result.updated, 1);

        let pulls = tracker_pulls_for_uid(uid, &pool)
            .await
            .expect("pulls should load");
        let stored = pulls
            .into_iter()
            .find(|pull| pull.record_uid == "repair-1")
            .expect("repaired pull should exist");

        assert_eq!(stored.reward_id, "1076");
        assert_eq!(stored.timestamp_raw, original_pull.timestamp_raw);
        assert_eq!(
            stored.timestamp_group_ordinal,
            original_pull.timestamp_group_ordinal
        );
        assert_eq!(stored.roll_result, Some(99));
        assert_eq!(stored.result_type, Some("dice".to_string()));
        assert_eq!(stored.quantity, Some(9));

        delete_test_user(&username, &pool).await;
    }

    #[actix_web::test]
    async fn reimporting_identical_pull_reports_no_insert_or_update() {
        let pool = test_pool().await;
        let (username, user_id) = create_test_user(&pool).await;
        let uid = next_uid();

        claim_tracker_uid(user_id, uid, 3, &pool)
            .await
            .expect("claim should succeed")
            .expect("claim should be created");

        let pull = sample_pull(uid, "identical-1");
        import_tracker_pulls(uid, std::slice::from_ref(&pull), 10, &pool)
            .await
            .expect("initial import should succeed")
            .expect("initial import should not hit the limit");

        let result = import_tracker_pulls(uid, &[pull], 10, &pool)
            .await
            .expect("duplicate import should succeed")
            .expect("duplicate import should not hit the limit");

        assert_eq!(result.inserted, 0);
        assert_eq!(result.updated, 0);
        assert_eq!(
            tracker_pull_count(uid, &pool)
                .await
                .expect("pull count should load"),
            1
        );

        delete_test_user(&username, &pool).await;
    }

    #[actix_web::test]
    async fn deleting_a_tracker_profile_removes_claim_and_pulls_atomically() {
        let pool = test_pool().await;
        let (username, user_id) = create_test_user(&pool).await;
        let uid = next_uid();

        claim_tracker_uid(user_id, uid, 3, &pool)
            .await
            .expect("claim should succeed")
            .expect("claim should be created");
        import_tracker_pulls(
            uid,
            &[sample_pull(uid, "delete-1"), sample_pull(uid, "delete-2")],
            10,
            &pool,
        )
        .await
        .expect("import should succeed")
        .expect("import should not hit the limit");

        let deleted_pulls = delete_tracker_uid_profile(user_id, uid, &pool)
            .await
            .expect("delete should succeed")
            .expect("delete should be allowed");

        assert_eq!(deleted_pulls, 2);
        assert!(get_tracker_claim(uid, &pool)
            .await
            .expect("claim lookup should succeed")
            .is_none());
        assert_eq!(
            tracker_pull_count(uid, &pool)
                .await
                .expect("pull count should load"),
            0
        );

        delete_test_user(&username, &pool).await;
    }
    #[sqlx::test]
    async fn sql_performance_tracker_bulk_replay_repairs_and_concurrent_cap(pool: PgPool) {
        let (_, user_id) = create_test_user(&pool).await;
        let uid = next_uid();
        claim_tracker_uid(user_id, uid, 3, &pool)
            .await
            .unwrap()
            .unwrap();
        // The same record ID under another UID is a separate history.
        let other_uid = next_uid();
        claim_tracker_uid(user_id, other_uid, 3, &pool)
            .await
            .unwrap()
            .unwrap();
        import_tracker_pulls(other_uid, &[sample_pull(other_uid, "seed-0")], 10, &pool)
            .await
            .unwrap()
            .unwrap();
        let mut pulls: Vec<_> = (0..10_000)
            .map(|i| sample_pull(uid, &format!("seed-{i}")))
            .collect();
        let first = import_tracker_pulls(uid, &pulls, 10_001, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            (first.inserted, first.updated, first.total),
            (10_000, 0, 10_000)
        );
        let replay = import_tracker_pulls(uid, &pulls, 10_001, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!((replay.inserted, replay.updated), (0, 0));
        pulls[0].quantity = None;
        pulls[0].result_type = None;
        pulls[0].roll_result = None;
        pulls[0].reward_id = "must-not-replace".into();
        pulls.push(sample_pull(uid, "new"));
        let mixed = import_tracker_pulls(uid, &pulls, 10_001, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!((mixed.inserted, mixed.updated, mixed.total), (1, 1, 10_001));
        let stored = tracker_pulls_for_uid(uid, &pool).await.unwrap();
        let repaired = stored.iter().find(|p| p.record_uid == "seed-0").unwrap();
        assert_eq!(repaired.quantity, None);
        assert_eq!(repaired.result_type, None);
        assert_eq!(repaired.roll_result, None);
        assert_eq!(repaired.reward_id, "test_reward");
        // An insert failure after a repair must roll the entire import back.
        let mut repair = pulls[0].clone();
        repair.quantity = Some(2);
        let mut invalid = sample_pull(uid, "invalid-new");
        invalid.timestamp_raw = "invalid".into();
        assert!(import_tracker_pulls(uid, &[repair, invalid], 20_000, &pool)
            .await
            .is_err());
        let stored = tracker_pulls_for_uid(uid, &pool).await.unwrap();
        assert_eq!(stored.len(), 10_001);
        assert_eq!(
            stored
                .iter()
                .find(|p| p.record_uid == "seed-0")
                .unwrap()
                .quantity,
            None
        );
        assert_eq!(
            tracker_pulls_for_uid(other_uid, &pool).await.unwrap()[0].quantity,
            Some(1)
        );
        // Two independent transactions compete for one remaining slot.
        let a = [sample_pull(uid, "concurrent-a")];
        let b = [sample_pull(uid, "concurrent-b")];
        let (a, b) = futures::join!(
            import_tracker_pulls(uid, &a, 10_002, &pool),
            import_tracker_pulls(uid, &b, 10_002, &pool)
        );
        assert_eq!(
            usize::from(a.unwrap().is_some()) + usize::from(b.unwrap().is_some()),
            1
        );
        assert_eq!(tracker_pull_count(uid, &pool).await.unwrap(), 10_002);
        let empty = import_tracker_pulls(uid, &[], 10_002, &pool)
            .await
            .unwrap()
            .unwrap();
        assert_eq!((empty.inserted, empty.updated, empty.total), (0, 0, 10_002));
    }
}
