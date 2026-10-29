//! Profiles and progress, saved as JSON in the user's data directory.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::i18n::Language;
use crate::theme::ThemeChoice;
use crate::typing::KeyStat;

const MAX_TESTS: usize = 500;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Kid-friendly words, shorter lessons, easier stars and cheerful messages.
    pub kids: bool,
    pub language: Language,
    pub relaxed_accents: bool,
    pub keyboard_in_test: bool,
    pub test_seconds: u64,
    pub theme: ThemeChoice,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            kids: false,
            language: Language::default(),
            relaxed_accents: false,
            keyboard_in_test: false,
            test_seconds: 30,
            theme: ThemeChoice::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct LessonRecord {
    pub stars: u8,
    pub best_wpm: f64,
    pub best_accuracy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TestRecord {
    pub wpm: f64,
    pub accuracy: f64,
    pub seconds: u64,
    pub language: Language,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub lessons: BTreeMap<usize, LessonRecord>,
    #[serde(default)]
    pub tests: Vec<TestRecord>,
    #[serde(default)]
    pub keys: BTreeMap<char, KeyStat>,
}

impl Profile {
    pub fn new(name: String, language: Language) -> Self {
        Self {
            name,
            settings: Settings { language, ..Settings::default() },
            lessons: BTreeMap::new(),
            tests: Vec::new(),
            keys: BTreeMap::new(),
        }
    }

    pub fn lesson_stars(&self, index: usize) -> u8 {
        self.lessons.get(&index).map_or(0, |record| record.stars)
    }

    pub fn is_unlocked(&self, index: usize) -> bool {
        index == 0 || self.lesson_stars(index - 1) > 0
    }

    pub fn total_stars(&self) -> u32 {
        self.lessons.values().map(|record| u32::from(record.stars)).sum()
    }

    pub fn best_wpm(&self) -> Option<f64> {
        self.tests.iter().map(|test| test.wpm).reduce(f64::max)
    }

    pub fn recent_average(&self, count: usize) -> Option<f64> {
        let recent = &self.tests[self.tests.len().saturating_sub(count)..];
        (!recent.is_empty()).then(|| recent.iter().map(|test| test.wpm).sum::<f64>() / recent.len() as f64)
    }

    /// Returns true when this attempt beat an earlier one.
    pub fn record_lesson(&mut self, index: usize, stars: u8, wpm: f64, accuracy: f64) -> bool {
        let previous = self.lessons.get(&index).copied();
        let record = self.lessons.entry(index).or_default();
        record.stars = record.stars.max(stars);
        record.best_accuracy = record.best_accuracy.max(accuracy);
        // A fast but sloppy attempt shouldn't set the speed record.
        if stars > 0 {
            record.best_wpm = record.best_wpm.max(wpm);
        }
        previous.is_some_and(|p| stars > p.stars || (stars > 0 && p.best_wpm > 0.0 && wpm > p.best_wpm))
    }

    /// Returns true when this beat the best earlier test of the same length and language.
    pub fn record_test(&mut self, record: TestRecord) -> bool {
        let previous_best = self
            .tests
            .iter()
            .filter(|test| test.seconds == record.seconds && test.language == record.language)
            .map(|test| test.wpm)
            .reduce(f64::max);
        self.tests.push(record);
        if self.tests.len() > MAX_TESTS {
            self.tests.remove(0);
        }
        previous_best.is_some_and(|best| record.wpm > best)
    }

    pub fn merge_keys(&mut self, stats: &BTreeMap<char, KeyStat>) {
        for (key, stat) in stats {
            let total = self.keys.entry(*key).or_default();
            total.total += stat.total;
            total.misses += stat.misses;
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Store {
    /// The language used before a profile is picked.
    #[serde(default)]
    pub language: Language,
    /// The theme used before a profile is picked.
    #[serde(default)]
    pub theme: ThemeChoice,
    #[serde(default)]
    pub last_profile: Option<String>,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    /// Set when existing data couldn't be read and couldn't be moved aside,
    /// so saving would destroy it.
    #[serde(skip)]
    read_only: bool,
}

impl Store {
    /// Loads saved data. Unreadable data is moved aside rather than
    /// overwritten; the returned message says what happened.
    pub fn load(path: &Path) -> (Store, Option<String>) {
        let error = match fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(store) => return (store, None),
                Err(e) => e.to_string(),
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => return (Store::default(), None),
            Err(e) => e.to_string(),
        };
        let backup = path.with_extension(format!("json.broken-{}", now()));
        match fs::rename(path, &backup) {
            Ok(()) => {
                let message = format!("Saved data was unreadable ({error}), so it was moved to {}", backup.display());
                (Store::default(), Some(message))
            }
            Err(_) => {
                let message = format!("Saved data is unreadable ({error}). Progress won't be saved this session");
                (Store { read_only: true, ..Store::default() }, Some(message))
            }
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if self.read_only {
            return Ok(());
        }
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        let temp = path.with_extension("json.tmp");
        fs::write(&temp, serde_json::to_string_pretty(self)?)?;
        fs::rename(&temp, path)
    }
}

/// `$KEYQUEST_DATA` if set, otherwise `keyquest/profiles.json` in the platform data directory.
pub fn data_path() -> PathBuf {
    if let Some(path) = std::env::var_os("KEYQUEST_DATA") {
        return PathBuf::from(path);
    }
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("keyquest").join("profiles.json")
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("keyquest-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("profiles.json")
    }

    fn test(wpm: f64, seconds: u64) -> TestRecord {
        TestRecord { wpm, accuracy: 95.0, seconds, language: Language::English, timestamp: 0 }
    }

    #[test]
    fn saves_and_loads_profiles() {
        let path = temp_path("roundtrip");
        let mut profile = Profile::new("Ana".into(), Language::Portuguese);
        profile.record_lesson(0, 2, 20.0, 95.0);
        profile.record_test(test(40.0, 30));
        profile.merge_keys(&BTreeMap::from([('á', KeyStat { total: 3, misses: 1 })]));
        let store = Store { profiles: vec![profile], ..Store::default() };
        store.save(&path).unwrap();

        let (loaded, warning) = Store::load(&path);
        assert!(warning.is_none());
        let loaded = &loaded.profiles[0];
        assert_eq!(loaded.name, "Ana");
        assert_eq!(loaded.settings.language, Language::Portuguese);
        assert_eq!(loaded.lesson_stars(0), 2);
        assert_eq!(loaded.keys[&'á'], KeyStat { total: 3, misses: 1 });
    }

    #[test]
    fn broken_data_is_moved_aside_not_overwritten() {
        let path = temp_path("broken");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{ not json").unwrap();
        let (store, warning) = Store::load(&path);
        assert!(store.profiles.is_empty());
        assert!(warning.unwrap().contains("moved"));
        assert!(!path.exists());
    }

    #[test]
    fn lessons_unlock_in_order() {
        let mut profile = Profile::new("Bo".into(), Language::English);
        assert!(profile.is_unlocked(0));
        assert!(!profile.is_unlocked(1));
        profile.record_lesson(0, 0, 50.0, 70.0);
        assert!(!profile.is_unlocked(1));
        assert_eq!(profile.lessons[&0].best_wpm, 0.0);
        profile.record_lesson(0, 1, 20.0, 88.0);
        assert!(profile.is_unlocked(1));
    }

    #[test]
    fn personal_bests_compare_tests_of_the_same_length() {
        let mut profile = Profile::new("Cy".into(), Language::English);
        assert!(!profile.record_test(test(40.0, 30)));
        assert!(!profile.record_test(test(60.0, 60)));
        assert!(profile.record_test(test(45.0, 30)));
        assert!(!profile.record_test(test(44.0, 30)));
    }
}
