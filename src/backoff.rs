//! Process-wide cooldown for an upstream API after rate-limit responses.
//!
//! The first 429 pauses calls for [`INITIAL_PAUSE`]; each further 429 after a pause
//! ends doubles it up to [`MAX_PAUSE`], and any success resets it. A `Retry-After`
//! header, when present, replaces the computed pause.

use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

const INITIAL_PAUSE: Duration = Duration::from_secs(30);
const MAX_PAUSE: Duration = Duration::from_secs(10 * 60);
/// Upper bound for a server-provided `Retry-After`, so a bogus value cannot stall us for days.
const MAX_RETRY_AFTER: Duration = Duration::from_secs(60 * 60);

pub struct Backoff {
    name: &'static str,
    state: Mutex<State>,
}

struct State {
    until: Option<Instant>,
    next_pause: Duration,
}

impl Backoff {
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            state: Mutex::new(State {
                until: None,
                next_pause: INITIAL_PAUSE,
            }),
        }
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Time left in the current cooldown, or None when calls are allowed.
    pub fn remaining(&self) -> Option<Duration> {
        self.remaining_at(Instant::now())
    }

    fn remaining_at(&self, now: Instant) -> Option<Duration> {
        self.state()
            .until
            .and_then(|until| until.checked_duration_since(now))
            .filter(|d| !d.is_zero())
    }

    /// Starts a cooldown after a 429. Responses to requests already in flight during an
    /// active cooldown do not extend or escalate it.
    pub fn rate_limited(&self, retry_after: Option<Duration>) {
        self.rate_limited_at(Instant::now(), retry_after);
    }

    fn rate_limited_at(&self, now: Instant, retry_after: Option<Duration>) {
        let mut state = self.state();
        if state.until.is_some_and(|until| until > now) {
            return;
        }

        let pause = retry_after
            .map(|d| d.min(MAX_RETRY_AFTER))
            .unwrap_or(state.next_pause);
        state.until = Some(now + pause);
        state.next_pause = (state.next_pause * 2).min(MAX_PAUSE);

        warn!(
            upstream = self.name,
            pause_secs = pause.as_secs(),
            "rate limited, pausing requests"
        );
    }

    /// Clears any cooldown and resets the escalation after a successful response.
    pub fn succeeded(&self) {
        let mut state = self.state();
        if state.next_pause != INITIAL_PAUSE {
            info!(
                upstream = self.name,
                "rate limit cleared, resuming requests"
            );
        }
        state.until = None;
        state.next_pause = INITIAL_PAUSE;
    }
}

/// Parses a `Retry-After` header given in seconds; HTTP-date values are ignored.
pub fn retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_doubles_until_success_and_caps() {
        let backoff = Backoff::new("test");
        let mut now = Instant::now();
        assert_eq!(backoff.remaining_at(now), None);

        let mut seen = Vec::new();
        for _ in 0..7 {
            backoff.rate_limited_at(now, None);
            let pause = backoff.remaining_at(now).unwrap();
            seen.push(pause.as_secs());
            now += pause;
            assert_eq!(backoff.remaining_at(now), None);
        }
        assert_eq!(seen, [30, 60, 120, 240, 480, 600, 600]);

        backoff.succeeded();
        backoff.rate_limited_at(now, None);
        assert_eq!(backoff.remaining_at(now), Some(INITIAL_PAUSE));
    }

    #[test]
    fn in_flight_429s_do_not_escalate() {
        let backoff = Backoff::new("test");
        let now = Instant::now();
        backoff.rate_limited_at(now, None);
        backoff.rate_limited_at(now + Duration::from_secs(1), None);
        assert_eq!(backoff.remaining_at(now), Some(INITIAL_PAUSE));

        backoff.rate_limited_at(now + INITIAL_PAUSE, None);
        assert_eq!(
            backoff.remaining_at(now + INITIAL_PAUSE),
            Some(INITIAL_PAUSE * 2)
        );
    }

    #[test]
    fn retry_after_overrides_and_is_capped() {
        let backoff = Backoff::new("test");
        let now = Instant::now();
        backoff.rate_limited_at(now, Some(Duration::from_secs(5)));
        assert_eq!(backoff.remaining_at(now), Some(Duration::from_secs(5)));

        let backoff = Backoff::new("test");
        backoff.rate_limited_at(now, Some(Duration::from_secs(999_999)));
        assert_eq!(backoff.remaining_at(now), Some(MAX_RETRY_AFTER));
    }

    #[test]
    fn parses_retry_after_seconds_only() {
        let mut headers = reqwest::header::HeaderMap::new();
        assert_eq!(retry_after(&headers), None);
        headers.insert(reqwest::header::RETRY_AFTER, "120".parse().unwrap());
        assert_eq!(retry_after(&headers), Some(Duration::from_secs(120)));
        headers.insert(
            reqwest::header::RETRY_AFTER,
            "Wed, 21 Oct 2015 07:28:00 GMT".parse().unwrap(),
        );
        assert_eq!(retry_after(&headers), None);
    }
}
