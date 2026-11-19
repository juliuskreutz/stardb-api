//! Pure chronological aggregate scans; tracker row annotation remains separate.
use crate::gacha::{
    banner::{BannerOutcome, GuaranteeState, GuaranteedOutcome},
    stats_math::average_or_zero,
};
#[derive(Default, Debug, PartialEq)]
pub(super) struct Scan {
    pub low: f64,
    pub high: f64,
    pub win_rate: f64,
    pub win_streak: i32,
    pub loss_streak: i32,
}
/// Computes completed pity intervals without a banner win/loss model.
/// `skip_first_high` preserves ZZZ Stable Channel's special first S-rank exclusion.
pub(super) fn scan_pity<T>(
    rows: &[T],
    rarity: impl Fn(&T) -> i32,
    low: i32,
    high: i32,
    skip_first_high: bool,
) -> Scan {
    scan_event(rows, rarity, |_| None, low, high, skip_first_high)
}
/// Scans rows in the supplied order; callers use ascending database pull IDs.
/// Incomplete trailing intervals are not averaged, and this function does not sort.
/// The outcome callback runs only for high-rarity pulls. None marks a pool with
/// no win model, while guaranteed results still contribute to high-rarity pity.
pub(super) fn scan_event<T>(
    rows: &[T],
    rarity: impl Fn(&T) -> i32,
    outcome: impl Fn(&T) -> Option<BannerOutcome>,
    low: i32,
    high: i32,
    mut skip_first_high: bool,
) -> Scan {
    let (mut pull_low, mut pull_high, mut sum_low, mut sum_high, mut count_low, mut count_high) =
        (0, 0, 0, 0, 0, 0);
    let (mut wins, mut decisions, mut win_streak, mut loss_streak) = (0, 0, 0, 0);
    let mut state = GuaranteeState::default();
    let mut result = Scan::default();
    for row in rows {
        pull_low += 1;
        pull_high += 1;
        let rank = rarity(row);
        // Aggregate low/high intervals reset independently, including in ZZZ.
        // ZZZ's tracker display resets A-rank on S-rank too; that is a different policy.
        if rank == low {
            sum_low += pull_low;
            count_low += 1;
            pull_low = 0;
        } else if rank == high {
            // Discard only the first completed high interval, then start measuring
            // from that pull. Do not clear the independent low-rarity interval.
            if skip_first_high {
                skip_first_high = false;
                pull_high = 0;
                continue;
            }
            sum_high += pull_high;
            count_high += 1;
            pull_high = 0;
            if let Some(outcome) = outcome(row) {
                match state.advance(outcome) {
                    // Guaranteed pulls neither enter the win-rate denominator nor
                    // break/extend confirmed decision streaks. Trackers label them
                    // explicitly, but stats measure only non-guaranteed decisions.
                    GuaranteedOutcome::GuaranteedWin => {}
                    GuaranteedOutcome::Win => {
                        decisions += 1;
                        wins += 1;
                        loss_streak = 0;
                        win_streak += 1;
                        result.win_streak = result.win_streak.max(win_streak);
                    }
                    GuaranteedOutcome::Loss => {
                        decisions += 1;
                        win_streak = 0;
                        loss_streak += 1;
                        result.loss_streak = result.loss_streak.max(loss_streak);
                    }
                }
            }
        }
    }
    result.low = average_or_zero(sum_low, count_low);
    result.high = average_or_zero(sum_high, count_high);
    result.win_rate = average_or_zero(wins, decisions);
    result
}
