//! Shared tracker grouping and annotation; payloads remain generic over each game.
use crate::Language;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
};

pub trait Item {
    fn id(&self) -> i32;
    fn currency(&self) -> i32;
    fn set_series_index(&mut self, index: usize);
}
pub struct Entry<A> {
    pub achievement: A,
    pub set: Option<i32>,
    pub series_name: String,
    pub version: Option<String>,
    pub hidden: bool,
    pub impossible: bool,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Tracker<A, E> {
    pub achievement_count: usize,
    pub achievement_count_current: usize,
    pub currency_count: i32,
    pub currency_count_current: i32,
    #[serde(flatten)]
    pub extra: E,
    pub language: Language,
    pub versions: Vec<String>,
    pub series: Vec<Series<A>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Series<A> {
    pub series: String,
    pub achievement_count: usize,
    pub achievement_count_current: usize,
    pub currency_count: i32,
    pub currency_count_current: i32,
    pub achievement_groups: Vec<AchievementGroup<A>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AchievementGroup<A> {
    pub complete: Option<i32>,
    pub favorite: Option<i32>,
    pub achievements: Vec<A>,
}

/// Consume each adapter's catalog order: series and members of a set must be contiguous.
/// Keeping that order preserves display ordering and the first member's currency total.
/// Visibility is a game policy: HSR includes hidden impossible entries; GI/ZZZ exclude
/// them before grouping. The generic payload retains each game's distinct JSON fields.
pub fn build<A: Item, E>(
    achievements: Vec<Entry<A>>,
    language: Language,
    extra: E,
    filter_hidden_impossible: bool,
) -> Tracker<A, E> {
    let mut versions = HashSet::new();
    let mut series = Vec::new();

    let mut current_series = None;
    let mut current_set = None;

    for achievement in achievements
        .into_iter()
        .filter(|a| !(filter_hidden_impossible && a.hidden && a.impossible))
    {
        versions.insert(achievement.version.clone().unwrap_or_default());

        if current_series != Some(achievement.series_name.clone()) {
            current_series = Some(achievement.series_name.clone());
            current_set = None;

            series.push(Series {
                series: achievement.series_name.clone(),
                achievement_count: 0,
                achievement_count_current: 0,
                currency_count: 0,
                currency_count_current: 0,
                achievement_groups: Vec::new(),
            });
        }

        if achievement
            .set
            .map(|set| current_set == Some(set))
            .unwrap_or(false)
        {
            let mut achievement = achievement.achievement;
            achievement.set_series_index(series.len() - 1);

            series
                .last_mut()
                .unwrap()
                .achievement_groups
                .last_mut()
                .unwrap()
                .achievements
                .push(achievement);
        } else {
            current_set = achievement.set;

            let mut achievement = achievement.achievement;
            achievement.set_series_index(series.len() - 1);

            series
                .last_mut()
                .unwrap()
                .achievement_groups
                .push(AchievementGroup {
                    complete: None,
                    favorite: None,
                    achievements: vec![achievement],
                });
        }
    }

    let mut achievement_count = 0;
    let mut currency_count = 0;

    for series in series.iter_mut() {
        series.achievement_count = series.achievement_groups.len();
        series.currency_count = series
            .achievement_groups
            .iter()
            .map(|group| group.achievements[0].currency())
            .sum();

        achievement_count += series.achievement_count;
        currency_count += series.currency_count;
    }

    let mut versions = versions.into_iter().collect::<Vec<_>>();
    versions.sort_unstable();

    Tracker {
        achievement_count,
        achievement_count_current: 0,
        currency_count,
        currency_count_current: 0,
        extra,
        language,
        versions,
        series,
    }
}
impl<A: Item, E> Tracker<A, E> {
    pub fn annotate(&mut self, completed: &HashSet<i32>, favorites: &HashSet<i32>) {
        let mut achievement_count_current_total = 0;
        let mut currency_count_current_total = 0;

        for series in self.series.iter_mut() {
            let mut achievement_count_current = 0;
            let mut currency_count_current = 0;

            for group in series.achievement_groups.iter_mut() {
                let complete = group
                    .achievements
                    .iter()
                    .map(|c| c.id())
                    .find(|id| completed.contains(id));

                let favorite = group
                    .achievements
                    .iter()
                    .map(|c| c.id())
                    .find(|id| favorites.contains(id));

                group.complete = complete;
                group.favorite = favorite;

                if let Some(complete) = complete {
                    achievement_count_current += 1;

                    currency_count_current += group
                        .achievements
                        .iter()
                        .find(|a| a.id() == complete)
                        .unwrap()
                        .currency();
                }
            }

            series.achievement_count_current = achievement_count_current;
            series.currency_count_current = currency_count_current;

            achievement_count_current_total += achievement_count_current;
            currency_count_current_total += currency_count_current;
        }

        self.achievement_count_current = achievement_count_current_total;
        self.currency_count_current = currency_count_current_total;
    }
}

pub fn load<T: DeserializeOwned>(path: &str) -> HashMap<Language, T> {
    File::open(path)
        .ok()
        .and_then(|file| serde_json::from_reader(BufReader::new(file)).ok())
        .unwrap_or_default()
}
pub fn save<T: Serialize>(path: &str, map: &HashMap<Language, T>) -> anyhow::Result<()> {
    std::fs::create_dir_all("cache")?;
    let temporary = format!("{path}.tmp");
    std::fs::write(&temporary, serde_json::to_vec(map)?)?;
    std::fs::rename(temporary, path)?;
    Ok(())
}

/// Refresh every language before replacing either the persisted or live cache.
pub async fn refresh<A, E, D, F, Fut, C>(
    cache: &async_rwlock::RwLock<HashMap<Language, Tracker<A, E>>>,
    pool: sqlx::PgPool,
    path: &str,
    extra: E,
    filter_hidden_impossible: bool,
    fetch: F,
    convert: C,
) -> anyhow::Result<()>
where
    A: Item + Serialize,
    E: Clone + Serialize,
    F: Fn(Language, sqlx::PgPool) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<Vec<D>>>,
    C: Fn(D) -> Entry<A>,
{
    use strum::IntoEnumIterator;
    let mut map = HashMap::new();
    for language in Language::iter() {
        let entries = fetch(language, pool.clone())
            .await?
            .into_iter()
            .map(&convert)
            .collect();
        map.insert(
            language,
            build(entries, language, extra.clone(), filter_hidden_impossible),
        );
    }
    save(path, &map)?;
    *cache.write().await = map;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone, Serialize, Deserialize)]
    struct TestItem {
        id: i32,
        currency: i32,
        series_index: usize,
    }
    impl Item for TestItem {
        fn id(&self) -> i32 {
            self.id
        }
        fn currency(&self) -> i32 {
            self.currency
        }
        fn set_series_index(&mut self, index: usize) {
            self.series_index = index;
        }
    }
    #[derive(Serialize)]
    struct Hsr {
        user_count: i64,
    }
    #[derive(Serialize)]
    struct Other {}
    fn entries() -> Vec<Entry<TestItem>> {
        [(1, "A", false), (2, "A", false), (3, "B", true)]
            .into_iter()
            .map(|(id, series, hidden)| Entry {
                achievement: TestItem {
                    id,
                    currency: id * 5,
                    series_index: 0,
                },
                set: Some(10),
                series_name: series.into(),
                version: Some("1.0".into()),
                hidden,
                impossible: hidden,
            })
            .collect()
    }
    #[test]
    fn grouping_boundary_annotation_and_game_wire_fields() {
        let mut hsr = build(entries(), Language::En, Hsr { user_count: 42 }, false);
        assert_eq!(hsr.achievement_count, 2);
        assert_eq!(
            hsr.series[1].achievement_groups[0].achievements[0].series_index,
            1
        );
        hsr.annotate(&HashSet::from([2]), &HashSet::from([1]));
        assert_eq!(hsr.achievement_count_current, 1);
        assert_eq!(hsr.currency_count_current, 10);
        assert_eq!(hsr.series[0].achievement_groups[0].favorite, Some(1));
        let value = serde_json::to_value(hsr).unwrap();
        let keys: HashSet<_> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            HashSet::from([
                "achievement_count",
                "achievement_count_current",
                "currency_count",
                "currency_count_current",
                "user_count",
                "language",
                "versions",
                "series"
            ])
        );
        assert_eq!(value["user_count"], 42);
        let other = build(entries(), Language::En, Other {}, true);
        assert_eq!(other.achievement_count, 1);
        assert_eq!(other.series.len(), 1);
        assert!(serde_json::to_value(other)
            .unwrap()
            .get("user_count")
            .is_none());
    }
}
