//! Frozen pre-refactor tracker JSON reference, extracted from 49daa02.
use super::*;
fn classify_win(
    catalog: &BannerCatalog,
    pool: ZzzGachaType,
    item: PullItem,
    timestamp: DateTime<Utc>,
    guarantee: &mut bool,
) -> WinType {
    match catalog.classify(PullPool::Zzz(pool), item, timestamp) {
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

