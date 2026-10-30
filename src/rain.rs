//! Word Rain: words fall from the sky, and typing one pops it before it lands.
//!
//! Positions are fractions of the sky, so the game plays the same at any
//! window size: `x` runs from 0 (left) to 1 (right), `y` from 0 (top) to 1
//! (the ground).

use crate::lessons::drill;
use crate::rng::Rng;
use crate::typing::matches;

pub const HEARTS: u8 = 3;
/// Words to pop before the rain speeds up.
pub const LEVEL_WORDS: u32 = 10;
/// How long pops and splashes stay on screen, in seconds.
pub const SPLASH_SECONDS: f64 = 0.7;
/// How long a word flashes after a wrong key, in seconds.
pub const FLASH_SECONDS: f64 = 0.3;
const MAX_LENGTH: usize = 9;
/// With fewer words than this, letter drills fill the pool.
const MIN_POOL: usize = 12;

pub struct Drop {
    pub word: Vec<char>,
    pub x: f64,
    pub y: f64,
    /// Letters typed so far. Only the word being typed has any.
    pub typed: usize,
    /// Seconds left of the flash after a wrong key.
    pub flash: f64,
}

/// Where a word popped, or landed when `points` is zero.
pub struct Splash {
    pub x: f64,
    pub y: f64,
    pub points: u32,
    pub age: f64,
}

pub struct Rain {
    pool: Vec<String>,
    relaxed_accents: bool,
    pub drops: Vec<Drop>,
    pub splashes: Vec<Splash>,
    pub score: u32,
    pub popped: u32,
    pub hearts: u8,
    keystrokes: u32,
    misses: u32,
    started: bool,
    /// Seconds until the next word appears.
    spawn_in: f64,
}

/// Words from `words` that can be typed with `keys`, topped up with drills
/// on those keys when there aren't enough.
pub fn pool(words: &[&str], keys: &[char], rng: &mut Rng) -> Vec<String> {
    let letters: Vec<char> = keys.iter().copied().filter(|c| c.is_alphabetic()).collect();
    let mut pool: Vec<String> = words
        .iter()
        .filter(|word| (2..=MAX_LENGTH).contains(&word.chars().count()) && word.chars().all(|c| letters.contains(&c)))
        .map(|word| word.to_string())
        .collect();
    while pool.len() < MIN_POOL {
        pool.push(drill(&letters, &letters, rng));
    }
    pool
}

impl Rain {
    pub fn new(pool: Vec<String>, relaxed_accents: bool) -> Self {
        Self {
            pool,
            relaxed_accents,
            drops: Vec::new(),
            splashes: Vec::new(),
            score: 0,
            popped: 0,
            hearts: HEARTS,
            keystrokes: 0,
            misses: 0,
            started: false,
            spawn_in: 0.0,
        }
    }

    pub fn start(&mut self) {
        self.started = true;
    }

    pub fn is_started(&self) -> bool {
        self.started
    }

    pub fn is_over(&self) -> bool {
        self.hearts == 0
    }

    pub fn level(&self) -> u32 {
        1 + self.popped / LEVEL_WORDS
    }

    /// Seconds a word takes to fall all the way down.
    fn fall_seconds(&self) -> f64 {
        (11.0 * 0.88_f64.powi(self.level() as i32 - 1)).max(3.5)
    }

    fn spawn_seconds(&self) -> f64 {
        (3.2 * 0.88_f64.powi(self.level() as i32 - 1)).max(1.0)
    }

    pub fn accuracy(&self) -> f64 {
        if self.keystrokes == 0 {
            return 100.0;
        }
        100.0 * f64::from(self.keystrokes - self.misses) / f64::from(self.keystrokes)
    }

    /// Moves the game on by `seconds`.
    pub fn tick(&mut self, seconds: f64, rng: &mut Rng) {
        if !self.started || self.is_over() {
            return;
        }
        for splash in &mut self.splashes {
            splash.age += seconds;
        }
        self.splashes.retain(|splash| splash.age < SPLASH_SECONDS);

        let fall = seconds / self.fall_seconds();
        for drop in &mut self.drops {
            drop.y += fall;
            drop.flash = (drop.flash - seconds).max(0.0);
        }
        let (landed, falling) = std::mem::take(&mut self.drops).into_iter().partition(|drop| drop.y >= 1.0);
        self.drops = falling;
        for drop in landed {
            self.hearts = self.hearts.saturating_sub(1);
            self.splashes.push(Splash { x: drop.x, y: 1.0, points: 0, age: 0.0 });
        }
        if self.is_over() {
            return;
        }

        self.spawn_in -= seconds;
        if self.spawn_in <= 0.0 || self.drops.is_empty() {
            self.spawn(rng);
            self.spawn_in = self.spawn_seconds();
        }
    }

    fn spawn(&mut self, rng: &mut Rng) {
        // Prefer a word that doesn't start like one already falling, so the
        // first letter picks the word.
        let mut word = rng.pick(&self.pool).clone();
        for _ in 0..10 {
            let first = word.chars().next();
            if !self.drops.iter().any(|drop| drop.word.first().copied() == first) {
                break;
            }
            word = rng.pick(&self.pool).clone();
        }
        // Keep away from words near the top, which could overlap the new one.
        let x = (0..8)
            .map(|_| rng.below(1001) as f64 / 1000.0)
            .max_by(|a, b| self.room(*a).total_cmp(&self.room(*b)))
            .unwrap_or(0.5);
        self.drops.push(Drop { word: word.chars().collect(), x, y: 0.0, typed: 0, flash: 0.0 });
    }

    /// How far `x` is from the closest word still near the top.
    fn room(&self, x: f64) -> f64 {
        self.drops.iter().filter(|drop| drop.y < 0.3).map(|drop| (drop.x - x).abs()).fold(1.0, f64::min)
    }

    /// The word being typed, if any.
    pub fn target(&self) -> Option<usize> {
        self.drops.iter().position(|drop| drop.typed > 0)
    }

    /// Types `c` into the word in progress, or starts the lowest word that
    /// begins with it. A wrong key starts the word in progress over, so
    /// mashing keys doesn't pay. Spaces are ignored, since words pop on their
    /// last letter.
    pub fn type_char(&mut self, c: char) {
        if !self.started || self.is_over() || c == ' ' {
            return;
        }
        self.keystrokes += 1;
        let relaxed = self.relaxed_accents;
        let hit = match self.target() {
            Some(i) => {
                let drop = &self.drops[i];
                matches(c, drop.word[drop.typed], relaxed).then_some(i)
            }
            None => self
                .drops
                .iter()
                .enumerate()
                .filter(|(_, drop)| matches(c, drop.word[0], relaxed))
                .max_by(|a, b| a.1.y.total_cmp(&b.1.y))
                .map(|(i, _)| i),
        };
        let Some(i) = hit else {
            self.misses += 1;
            if let Some(i) = self.target() {
                self.drops[i].typed = 0;
                self.drops[i].flash = FLASH_SECONDS;
            }
            return;
        };
        let drop = &mut self.drops[i];
        drop.typed += 1;
        if drop.typed == drop.word.len() {
            let drop = self.drops.remove(i);
            let points = drop.word.len() as u32 * self.level();
            self.score += points;
            self.popped += 1;
            self.splashes.push(Splash { x: drop.x, y: drop.y, points, age: 0.0 });
        }
    }

    pub fn backspace(&mut self) {
        if let Some(i) = self.target() {
            self.drops[i].typed -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(words: &[&str]) -> Rain {
        let mut rain = Rain::new(words.iter().map(|w| w.to_string()).collect(), false);
        rain.start();
        rain
    }

    fn drop(word: &str, y: f64) -> Drop {
        Drop { word: word.chars().collect(), x: 0.5, y, typed: 0, flash: 0.0 }
    }

    #[test]
    fn the_pool_only_uses_known_keys() {
        let pool = pool(&["fad", "ask", "dad", "jug", "a"], &['a', 's', 'd', 'f', 'j', 'k', 'l', ';'], &mut Rng::new(1));
        assert!(pool.contains(&"fad".to_string()) && pool.contains(&"ask".to_string()));
        assert!(!pool.iter().any(|w| w == "jug" || w == "a"));
        assert!(pool.len() >= MIN_POOL);
        for word in &pool {
            assert!(word.chars().all(|c| "asdfjkl".contains(c)), "{word}");
        }
    }

    #[test]
    fn typing_a_word_pops_it() {
        let mut rain = game(&["cat"]);
        rain.drops = vec![drop("dog", 0.2), drop("cat", 0.5)];
        for c in "cat".chars() {
            rain.type_char(c);
        }
        assert_eq!(rain.drops.len(), 1);
        assert_eq!(rain.popped, 1);
        assert_eq!(rain.score, 3);
        assert_eq!(rain.accuracy(), 100.0);
    }

    #[test]
    fn the_lowest_matching_word_is_picked() {
        let mut rain = game(&["sun"]);
        rain.drops = vec![drop("sea", 0.2), drop("sun", 0.7)];
        rain.type_char('s');
        assert_eq!(rain.target(), Some(1));
        rain.type_char('u');
        rain.backspace();
        assert_eq!(rain.drops[1].typed, 1);
    }

    #[test]
    fn a_wrong_key_starts_the_word_over() {
        let mut rain = game(&["sun"]);
        rain.drops = vec![drop("sun", 0.5)];
        rain.type_char('s');
        rain.type_char('u');
        rain.type_char('x');
        assert_eq!(rain.target(), None);
        assert!(rain.drops[0].flash > 0.0);
        assert!(rain.accuracy() < 100.0);
        // Mashing the right letters out of order never pops the word.
        for c in "nusnus".chars() {
            rain.type_char(c);
        }
        assert_eq!(rain.popped, 0);
        rain.tick(1.0, &mut Rng::new(1));
        assert_eq!(rain.drops[0].flash, 0.0);
    }

    #[test]
    fn landing_words_cost_hearts_until_the_game_ends() {
        let mut rain = game(&["owl"]);
        let mut rng = Rng::new(3);
        rain.tick(0.1, &mut rng);
        assert_eq!(rain.drops.len(), 1);
        for _ in 0..200 {
            rain.tick(0.5, &mut rng);
        }
        assert!(rain.is_over());
        assert_eq!(rain.hearts, 0);
        rain.type_char('o');
        assert_eq!(rain.target(), None);
    }

    #[test]
    fn nothing_falls_before_the_start() {
        let mut rain = Rain::new(vec!["owl".into()], false);
        rain.tick(5.0, &mut Rng::new(1));
        assert!(rain.drops.is_empty());
    }

    #[test]
    fn the_rain_speeds_up_with_each_level() {
        let mut rain = game(&["owl"]);
        let slow = rain.fall_seconds();
        rain.popped = LEVEL_WORDS;
        assert_eq!(rain.level(), 2);
        assert!(rain.fall_seconds() < slow);
        rain.popped = 1000;
        assert_eq!(rain.fall_seconds(), 3.5);
    }
}
