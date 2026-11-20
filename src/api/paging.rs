//! Page bounds shared by the achievement leaderboard endpoints.

/// Rows returned when the caller gives no limit.
pub(crate) const DEFAULT_LIMIT: i64 = 100;
/// Largest page a caller may request; larger requests are rejected, never truncated.
pub(crate) const MAX_LIMIT: i64 = 1_000;

/// Resolve a leaderboard page to `(limit, offset)`.
///
/// No limit means [`DEFAULT_LIMIT`] and no offset means 0. A limit outside
/// 1..=[`MAX_LIMIT`] or a negative offset is an error with a message for the caller,
/// so a client can never mistake a shortened page for the whole result.
pub(crate) fn leaderboard_page(
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<(i64, i64), String> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(format!("limit must be between 1 and {MAX_LIMIT}"));
    }
    let offset = offset.unwrap_or(0);
    if offset < 0 {
        return Err("offset must not be negative".to_owned());
    }
    Ok((limit, offset))
}

#[cfg(test)]
mod tests {
    use super::leaderboard_page;

    #[test]
    fn defaults_bounds_and_rejections() {
        assert_eq!(leaderboard_page(None, None), Ok((100, 0)));
        assert_eq!(leaderboard_page(Some(1), Some(20)), Ok((1, 20)));
        assert_eq!(leaderboard_page(Some(1_000), None), Ok((1_000, 0)));
        assert!(leaderboard_page(Some(0), None).is_err());
        assert!(leaderboard_page(Some(-5), None).is_err());
        assert!(leaderboard_page(Some(1_001), None).is_err());
        assert!(leaderboard_page(None, Some(-1)).is_err());
    }
}
