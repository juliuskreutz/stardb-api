use std::{collections::HashMap, ops::Range};

use chrono::{DateTime, Utc};

// Standard-pool 5★ HSR characters and light cones. A pull outside every configured banner window
// is a Loss only when the item is in this list; otherwise it is a Win.
pub const HSR_STANDARD: &[i32] = &[
    1209, 1004, 1101, 1211, 1104, 1107, 1003, // characters
    23000, 23002, 23003, 23004, 23005, 23012, 23013, // light cones
];

// Standard-pool 5★ Genshin characters and weapons.
pub const GI_STANDARD: &[i32] = &[
    10000042, 10000016, 10000003, 10000035, 10000069, 10000079, 10000041, // characters
    15502, 11501, 14502, 13505, 14501, 15501, 12501, 13502, 12502, // weapons
];

pub fn is_win_fn<'a>(
    banners: &'a HashMap<i32, Vec<Range<DateTime<Utc>>>>,
    standard: &'a [i32],
) -> impl Fn(i32, DateTime<Utc>) -> bool + 'a {
    move |item_id, timestamp| {
        let matching_items: Vec<_> = banners
            .iter()
            .filter(|(_, ranges)| ranges.iter().any(|range| range.contains(&timestamp)))
            .map(|(item_id, _)| *item_id)
            .collect();

        let is_standard = standard.contains(&item_id);
        let is_win = if matching_items.is_empty() {
            !is_standard
        } else {
            matching_items.contains(&item_id)
        };

        is_win
    }
}

#[cfg(test)]
mod banner_helpers_tests {
    use super::is_win_fn;
    use chrono::{TimeZone, Utc};
    use std::collections::HashMap;

    fn timestamp(hour: u32) -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, hour, 0, 0)
            .single()
            .expect("valid test timestamp")
    }

    #[test]
    fn banner_helpers_empty_catalog_treats_standard_item_as_loss() {
        let banners = HashMap::new();
        let classify = is_win_fn(&banners, &[10]);

        assert!(!classify(10, timestamp(1)));
    }

    #[test]
    fn banner_helpers_empty_catalog_treats_non_standard_item_as_win() {
        let banners = HashMap::new();
        let classify = is_win_fn(&banners, &[10]);

        assert!(classify(20, timestamp(1)));
    }

    #[test]
    fn banner_helpers_active_featured_item_is_a_win() {
        let banners = HashMap::from([(20, vec![timestamp(1)..timestamp(3)])]);
        let classify = is_win_fn(&banners, &[10]);

        assert!(classify(20, timestamp(2)));
    }

    #[test]
    fn banner_helpers_active_different_item_is_a_loss() {
        let banners = HashMap::from([(20, vec![timestamp(1)..timestamp(3)])]);
        let classify = is_win_fn(&banners, &[10]);

        assert!(!classify(30, timestamp(2)));
    }

    #[test]
    fn banner_helpers_ranges_include_start_and_exclude_end() {
        let banners = HashMap::from([(20, vec![timestamp(1)..timestamp(3)])]);
        let classify = is_win_fn(&banners, &[20]);

        assert!(classify(20, timestamp(1)));
        assert!(!classify(20, timestamp(3)));
    }

    #[test]
    fn banner_helpers_overlapping_ranges_accept_each_featured_item() {
        let banners = HashMap::from([
            (20, vec![timestamp(1)..timestamp(3)]),
            (30, vec![timestamp(2)..timestamp(4)]),
        ]);
        let classify = is_win_fn(&banners, &[10]);

        assert!(classify(20, timestamp(2)));
        assert!(classify(30, timestamp(2)));
    }
}
