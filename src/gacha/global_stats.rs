//! Shared population-percentile calculation for every gacha game.
//!
//! Database adapters remain game-specific because their tables and eligibility
//! rules differ. Once those adapters have produced the four values below, the
//! ranking formula is identical for HSR, Genshin, and ZZZ.

/// Maximum rows written by one bulk upsert.
pub(crate) const UPDATE_BATCH_SIZE: usize = 1_000;

/// Per-user values needed to calculate population-relative ranks.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PercentileInput {
    pub(crate) uid: i32,
    pub(crate) count: i32,
    pub(crate) luck_low: f64,
    pub(crate) luck_high: f64,
}

/// Zero-based percentile values persisted for one user.
#[derive(Debug, PartialEq)]
pub(crate) struct PercentileResult {
    pub(crate) uid: i32,
    pub(crate) count: f64,
    pub(crate) luck_low: f64,
    pub(crate) luck_high: f64,
}

/// Ranks count from largest to smallest and luck from smallest to largest.
///
/// The percentile formula intentionally remains `rank / population_size`.
/// Equal values use UID as a stable tie-breaker; the previous `HashMap`-based
/// implementations assigned tied ranks nondeterministically.
pub(crate) fn calculate_percentiles(inputs: Vec<PercentileInput>) -> Vec<PercentileResult> {
    let mut by_count = inputs.clone();
    by_count.sort_unstable_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.uid.cmp(&right.uid))
    });

    let mut by_luck_low = inputs.clone();
    by_luck_low.sort_unstable_by(|left, right| {
        left.luck_low
            .total_cmp(&right.luck_low)
            .then_with(|| left.uid.cmp(&right.uid))
    });

    let mut by_luck_high = inputs;
    by_luck_high.sort_unstable_by(|left, right| {
        left.luck_high
            .total_cmp(&right.luck_high)
            .then_with(|| left.uid.cmp(&right.uid))
    });

    let population = by_count.len() as f64;
    let mut results = Vec::with_capacity(by_count.len());
    for value in by_count {
        let count_rank = results.len();
        let luck_low_rank = by_luck_low
            .binary_search_by(|candidate| {
                candidate
                    .luck_low
                    .total_cmp(&value.luck_low)
                    .then_with(|| candidate.uid.cmp(&value.uid))
            })
            .expect("ranked input remains present");
        let luck_high_rank = by_luck_high
            .binary_search_by(|candidate| {
                candidate
                    .luck_high
                    .total_cmp(&value.luck_high)
                    .then_with(|| candidate.uid.cmp(&value.uid))
            })
            .expect("ranked input remains present");
        results.push(PercentileResult {
            uid: value.uid,
            count: count_rank as f64 / population,
            luck_low: luck_low_rank as f64 / population,
            luck_high: luck_high_rank as f64 / population,
        });
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_all_metrics_in_their_existing_directions() {
        let values = calculate_percentiles(vec![
            PercentileInput {
                uid: 1,
                count: 100,
                luck_low: 8.0,
                luck_high: 70.0,
            },
            PercentileInput {
                uid: 2,
                count: 50,
                luck_low: 6.0,
                luck_high: 80.0,
            },
        ]);

        assert_eq!(
            values,
            vec![
                PercentileResult {
                    uid: 1,
                    count: 0.0,
                    luck_low: 0.5,
                    luck_high: 0.0,
                },
                PercentileResult {
                    uid: 2,
                    count: 0.5,
                    luck_low: 0.0,
                    luck_high: 0.5,
                },
            ]
        );
    }

    #[test]
    fn ties_use_uid_as_a_stable_secondary_order() {
        let values = calculate_percentiles(vec![
            PercentileInput {
                uid: 20,
                count: 100,
                luck_low: 10.0,
                luck_high: 20.0,
            },
            PercentileInput {
                uid: 10,
                count: 100,
                luck_low: 10.0,
                luck_high: 20.0,
            },
        ]);

        assert_eq!(values[0].uid, 10);
        assert_eq!(
            (values[0].count, values[0].luck_low, values[0].luck_high),
            (0.0, 0.0, 0.0)
        );
        assert_eq!(
            (values[1].count, values[1].luck_low, values[1].luck_high),
            (0.5, 0.5, 0.5)
        );
    }

    #[test]
    fn empty_population_produces_no_non_finite_values() {
        assert!(calculate_percentiles(Vec::new()).is_empty());
    }

    #[test]
    fn matches_the_reference_rank_formula_across_generated_populations() {
        for population in 1..=64_i32 {
            let inputs: Vec<_> = (0..population)
                .rev()
                .map(|index| PercentileInput {
                    uid: 10_000 + index,
                    count: 100 + index,
                    luck_low: ((index + 17) % population) as f64,
                    luck_high: (population - index) as f64,
                })
                .collect();
            let results = calculate_percentiles(inputs.clone());
            let denominator = population as f64;

            for result in results {
                let value = inputs.iter().find(|value| value.uid == result.uid).unwrap();
                let expected_count = inputs
                    .iter()
                    .filter(|other| {
                        other.count > value.count
                            || (other.count == value.count && other.uid < value.uid)
                    })
                    .count() as f64
                    / denominator;
                let expected_low = inputs
                    .iter()
                    .filter(|other| {
                        other.luck_low < value.luck_low
                            || (other.luck_low == value.luck_low && other.uid < value.uid)
                    })
                    .count() as f64
                    / denominator;
                let expected_high = inputs
                    .iter()
                    .filter(|other| {
                        other.luck_high < value.luck_high
                            || (other.luck_high == value.luck_high && other.uid < value.uid)
                    })
                    .count() as f64
                    / denominator;

                assert_eq!(result.count, expected_count);
                assert_eq!(result.luck_low, expected_low);
                assert_eq!(result.luck_high, expected_high);
            }
        }
    }
}
