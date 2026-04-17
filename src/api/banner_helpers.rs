use std::{collections::HashMap, ops::Range};

use chrono::{DateTime, Utc};

// Standard-pool 5★ HSR characters and light cones. A pull whose item is in this list and whose
// timestamp falls outside every rate-up banner is a Loss; everything else counts as a Win.
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
        banners
            .get(&item_id)
            .is_some_and(|v| v.iter().any(|r| r.contains(&timestamp)))
            || !standard.contains(&item_id)
    }
}
