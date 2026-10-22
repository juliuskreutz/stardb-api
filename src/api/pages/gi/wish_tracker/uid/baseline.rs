// Preserve original conversion/borrow syntax in this frozen reference implementation.
#![allow(clippy::useless_conversion, clippy::needless_borrow)]
#![allow(unused_variables)]
//! Frozen tracker JSON reference extracted from 49daa02, before the table rewrite.
use super::*;
use crate::gacha::banner::BannerOutcome;
fn classify_win(
    catalog: &BannerCatalog,
    pool: GiGachaType,
    item: PullItem,
    timestamp: DateTime<Utc>,
    guarantee: &mut bool,
) -> WinType {
    match catalog.classify(PullPool::Gi(pool), item, timestamp) {
        BannerOutcome::Win if *guarantee => {
            *guarantee = false;
            WinType::Guarantee
        }
        BannerOutcome::Win => WinType::Win,
        BannerOutcome::Loss if *guarantee => {
            *guarantee = false;
            WinType::Guarantee
        }
        BannerOutcome::Loss => {
            *guarantee = true;
            WinType::Loss
        }
    }
}

pub(super) fn beginner(rows: Vec<Wish>, banner_catalog: &BannerCatalog) -> Wishes {
    let mut beginner = Wishes::default();
    let mut beginner_pull = 0;
    let mut beginner_pull_4 = 0;
    let mut beginner_pull_5 = 0;

    for wish in rows {
        let mut wish: Wish = wish.into();

        beginner_pull += 1;
        beginner_pull_4 += 1;
        beginner_pull_5 += 1;

        wish.pull = beginner_pull;
        wish.pull_4 = beginner_pull_4;
        wish.pull_5 = beginner_pull_5;

        match wish.rarity {
            4 => beginner_pull_4 = 0,
            5 => {
                beginner_pull_5 = 0;
            }
            _ => {}
        }

        beginner.wishes.push(wish);
    }

    beginner.pull_4 = beginner_pull_4;
    beginner.max_pull_4 = 10;
    beginner.probability_4 = if beginner_pull_4 < 9 { 9.4 } else { 100.0 };

    beginner.pull_5 = beginner_pull_5;
    beginner.max_pull_5 = 90;
    beginner.probability_5 = if beginner_pull_5 < 89 {
        0.6 + 6.0 * beginner_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    beginner.count = beginner.wishes.len();
    beginner
}
pub(super) fn standard(rows: Vec<Wish>, banner_catalog: &BannerCatalog) -> Wishes {
    let mut standard = Wishes::default();
    let mut standard_pull = 0;
    let mut standard_pull_4 = 0;
    let mut standard_pull_5 = 0;

    for wish in rows {
        let mut wish: Wish = wish.into();

        standard_pull += 1;
        standard_pull_4 += 1;
        standard_pull_5 += 1;

        wish.pull = standard_pull;
        wish.pull_4 = standard_pull_4;
        wish.pull_5 = standard_pull_5;

        match wish.rarity {
            4 => standard_pull_4 = 0,
            5 => {
                standard_pull_5 = 0;
            }
            _ => {}
        }

        standard.wishes.push(wish);
    }

    standard.pull_4 = standard_pull_4;
    standard.max_pull_4 = 10;
    standard.probability_4 = if standard_pull_4 < 9 { 9.4 } else { 100.0 };

    standard.pull_5 = standard_pull_5;
    standard.max_pull_5 = 90;
    standard.probability_5 = if standard_pull_5 < 89 {
        0.6 + 6.0 * standard_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    standard.count = standard.wishes.len();
    standard
}
pub(super) fn character(rows: Vec<Wish>, banner_catalog: &BannerCatalog) -> Wishes {
    let mut character = Wishes::default();
    let mut character_pull = 0;
    let mut character_pull_4 = 0;
    let mut character_pull_5 = 0;
    let mut guarantee = false;

    for wish in rows {
        let mut wish: Wish = wish.into();

        character_pull += 1;
        character_pull_4 += 1;
        character_pull_5 += 1;

        wish.pull = character_pull;
        wish.pull_4 = character_pull_4;
        wish.pull_5 = character_pull_5;

        match wish.rarity {
            4 => character_pull_4 = 0,
            5 => {
                character_pull_5 = 0;
                wish.win = Some(classify_win(
                    &banner_catalog,
                    GiGachaType::Character,
                    PullItem::Character(wish.item_id),
                    wish.timestamp,
                    &mut guarantee,
                ));
            }
            _ => {}
        }

        character.wishes.push(wish);
    }

    character.pull_4 = character_pull_4;
    character.max_pull_4 = 10;
    character.probability_4 = if character_pull_4 < 9 { 9.4 } else { 100.0 };

    character.pull_5 = character_pull_5;
    character.max_pull_5 = 90;
    character.probability_5 = if character_pull_5 < 89 {
        0.6 + 6.0 * character_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    character.count = character.wishes.len();
    character
}
pub(super) fn weapon(rows: Vec<Wish>, banner_catalog: &BannerCatalog) -> Wishes {
    let mut weapon = Wishes::default();
    let mut weapon_pull = 0;
    let mut weapon_pull_4 = 0;
    let mut weapon_pull_5 = 0;
    let mut guarantee = false;

    for wish in rows {
        let mut wish: Wish = wish.into();

        weapon_pull += 1;
        weapon_pull_4 += 1;
        weapon_pull_5 += 1;

        wish.pull = weapon_pull;
        wish.pull_4 = weapon_pull_4;
        wish.pull_5 = weapon_pull_5;

        match wish.rarity {
            4 => weapon_pull_4 = 0,
            5 => {
                weapon_pull_5 = 0;
                wish.win = Some(classify_win(
                    &banner_catalog,
                    GiGachaType::Weapon,
                    PullItem::Weapon(wish.item_id),
                    wish.timestamp,
                    &mut guarantee,
                ));
            }
            _ => {}
        }

        weapon.wishes.push(wish);
    }

    weapon.pull_4 = weapon_pull_4;
    weapon.max_pull_4 = 10;
    weapon.probability_4 = if weapon_pull_4 < 9 { 9.4 } else { 100.0 };

    weapon.pull_5 = weapon_pull_5;
    weapon.max_pull_5 = 90;
    weapon.probability_5 = if weapon_pull_5 < 89 {
        0.6 + 6.0 * weapon_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    weapon.count = weapon.wishes.len();
    weapon
}
pub(super) fn chronicled(rows: Vec<Wish>, banner_catalog: &BannerCatalog) -> Wishes {
    let mut chronicled = Wishes::default();
    let mut chronicled_pull = 0;
    let mut chronicled_pull_4 = 0;
    let mut chronicled_pull_5 = 0;

    for wish in rows {
        let mut wish: Wish = wish.into();

        chronicled_pull += 1;
        chronicled_pull_4 += 1;
        chronicled_pull_5 += 1;

        wish.pull = chronicled_pull;
        wish.pull_4 = chronicled_pull_4;
        wish.pull_5 = chronicled_pull_5;

        match wish.rarity {
            4 => chronicled_pull_4 = 0,
            5 => {
                chronicled_pull_5 = 0;
            }
            _ => {}
        }

        chronicled.wishes.push(wish);
    }

    chronicled.pull_4 = chronicled_pull_4;
    chronicled.max_pull_4 = 10;
    chronicled.probability_4 = if chronicled_pull_4 < 9 { 9.4 } else { 100.0 };

    chronicled.pull_5 = chronicled_pull_5;
    chronicled.max_pull_5 = 90;
    chronicled.probability_5 = if chronicled_pull_5 < 89 {
        0.6 + 6.0 * chronicled_pull_5.saturating_sub(72) as f64
    } else {
        100.0
    };

    chronicled.count = chronicled.wishes.len();
    chronicled
}
