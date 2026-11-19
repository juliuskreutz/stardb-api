/// Returns an arithmetic mean while treating an empty sample as finite zero.
///
/// Stored statistics are serialized as JSON, so returning `NaN` for an empty
/// history would make an otherwise valid import response fail to serialize.
pub(crate) fn average_or_zero(sum: usize, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        sum as f64 / count as f64
    }
}

#[cfg(test)]
mod gacha_security {
    mod stats_math {
        use super::super::average_or_zero;

        #[test]
        fn zero_denominator_returns_finite_zero() {
            let average = average_or_zero(0, 0);

            assert_eq!(average, 0.0);
            assert!(average.is_finite());
        }
    }
}
