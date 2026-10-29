//! Application state and keyboard handling for every screen.

use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::celebrate::{self, Confetti};
use crate::i18n::{Language, Strings};
use crate::lessons::{self, LESSONS, unlocked_keys};
use crate::rain::{self, Rain};
use crate::rng::Rng;
use crate::storage::{self, Profile, Settings, Store, TestRecord};
use crate::typing::Session;
use crate::{ui, words};

pub const TEST_LENGTHS: [u64; 4] = [15, 30, 60, 120];
pub const MENU_ITEMS: usize = 7;
pub const SETTINGS_ITEMS: usize = 6;
/// Lessons are shown in columns of this many rows.
pub const LESSON_ROWS: usize = 8;
pub const LESSON_COLUMNS: usize = LESSONS.len().div_ceil(LESSON_ROWS);
const MAX_NAME: usize = 20;
/// Keys pressed right as a test ends are ignored on the results screen for this long.
const RESULTS_GRACE: Duration = Duration::from_millis(600);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Lesson(usize),
    Speed(u64),
}

pub struct Typing {
    pub mode: Mode,
    pub session: Session,
    /// The streak that last hit a milestone, and when.
    pub cheer: Option<(u32, Instant)>,
}

pub struct Results {
    pub mode: Mode,
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub correct: usize,
    pub errors: u32,
    pub seconds: f64,
    pub samples: Vec<(f64, f64)>,
    pub stars: u8,
    pub personal_best: bool,
    pub missed: Vec<char>,
    pub best_streak: u32,
    pub confetti: Option<Confetti>,
    shown_at: Instant,
}

impl Results {
    /// Time since the screen appeared, for its animations.
    pub fn age(&self) -> Duration {
        self.shown_at.elapsed()
    }
}

pub struct RainPlay {
    pub game: Rain,
    pub over: Option<GameOver>,
    /// When the game last moved on.
    last: Instant,
}

pub struct GameOver {
    pub personal_best: bool,
    pub confetti: Option<Confetti>,
    at: Instant,
}

impl GameOver {
    /// Time since the game ended, for its animations.
    pub fn age(&self) -> Duration {
        self.at.elapsed()
    }
}

pub enum Screen {
    Profiles { selected: usize, confirm_delete: bool },
    NewProfile { name: String, error: Option<&'static str> },
    Menu { selected: usize },
    Lessons { selected: usize },
    Typing(Typing),
    Results(Results),
    Rain(RainPlay),
    Stats,
    Settings { selected: usize },
}

pub struct App {
    pub store: Store,
    pub profile: Option<usize>,
    pub screen: Screen,
    pub notice: Option<String>,
    path: PathBuf,
    rng: Rng,
    quit: bool,
}

impl App {
    pub fn new(store: Store, path: PathBuf, notice: Option<String>) -> Self {
        let screen = if store.profiles.is_empty() {
            Screen::NewProfile { name: String::new(), error: None }
        } else {
            let last = store.last_profile.as_deref();
            let selected = store.profiles.iter().position(|p| Some(p.name.as_str()) == last).unwrap_or(0);
            Screen::Profiles { selected, confirm_delete: false }
        };
        Self { store, profile: None, screen, notice, path, rng: Rng::from_time(), quit: false }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.quit {
            terminal.draw(|frame| ui::draw(frame, self))?;
            if event::poll(Duration::from_millis(50))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.on_key(key);
            }
            self.tick(Instant::now());
        }
        Ok(())
    }

    pub fn current(&self) -> Option<&Profile> {
        self.profile.and_then(|i| self.store.profiles.get(i))
    }

    fn current_mut(&mut self) -> Option<&mut Profile> {
        self.profile.and_then(|i| self.store.profiles.get_mut(i))
    }

    pub fn settings(&self) -> Settings {
        self.current()
            .map_or(Settings { language: self.store.language, theme: self.store.theme, ..Settings::default() }, |p| p.settings)
    }

    pub fn language(&self) -> Language {
        self.settings().language
    }

    pub fn strings(&self) -> &'static Strings {
        self.language().strings()
    }

    fn save(&mut self) {
        if let Err(e) = self.store.save(&self.path) {
            self.notice = Some(self.strings().save_failed.replace("{err}", &e.to_string()));
        }
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.quit = true;
            return;
        }
        self.notice = None;
        match &self.screen {
            &Screen::Profiles { selected, confirm_delete } => self.profiles_key(key, selected, confirm_delete),
            Screen::NewProfile { .. } => self.new_profile_key(key),
            &Screen::Menu { selected } => self.menu_key(key, selected),
            &Screen::Lessons { selected } => self.lessons_key(key, selected),
            Screen::Typing(_) => self.typing_key(key),
            Screen::Results(_) => self.results_key(key),
            Screen::Rain(_) => self.rain_key(key),
            Screen::Stats => self.stats_key(key),
            &Screen::Settings { selected } => self.settings_key(key, selected),
        }
    }

    fn profiles_key(&mut self, key: KeyEvent, selected: usize, confirm_delete: bool) {
        let count = self.store.profiles.len();
        if confirm_delete {
            // "y" for English, "s" (sim) for Portuguese.
            if matches!(key.code, KeyCode::Char('y' | 's')) && selected < count {
                let removed = self.store.profiles.remove(selected);
                if self.store.last_profile.as_deref() == Some(removed.name.as_str()) {
                    self.store.last_profile = None;
                }
                self.save();
            }
            let selected = selected.min(self.store.profiles.len());
            self.screen = Screen::Profiles { selected, confirm_delete: false };
            return;
        }
        let selected = match key.code {
            KeyCode::Up | KeyCode::Char('k') => selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => (selected + 1).min(count),
            KeyCode::Enter if selected < count => return self.select_profile(selected),
            KeyCode::Enter | KeyCode::Char('n') => return self.open_new_profile(),
            KeyCode::Char('d') | KeyCode::Delete if selected < count => {
                self.screen = Screen::Profiles { selected, confirm_delete: true };
                return;
            }
            KeyCode::Char('l') => {
                self.store.language = self.store.language.next();
                self.save();
                selected
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.quit = true;
                return;
            }
            _ => return,
        };
        self.screen = Screen::Profiles { selected, confirm_delete: false };
    }

    fn open_new_profile(&mut self) {
        self.screen = Screen::NewProfile { name: String::new(), error: None };
    }

    fn open_profiles(&mut self) {
        let selected = self.profile.take().unwrap_or(0);
        self.screen = Screen::Profiles { selected, confirm_delete: false };
    }

    fn select_profile(&mut self, index: usize) {
        let profile = &self.store.profiles[index];
        self.store.last_profile = Some(profile.name.clone());
        self.store.language = profile.settings.language;
        self.store.theme = profile.settings.theme;
        self.profile = Some(index);
        self.save();
        self.screen = Screen::Menu { selected: 0 };
    }

    fn new_profile_key(&mut self, key: KeyEvent) {
        let strings = self.strings();
        let Screen::NewProfile { name, error } = &mut self.screen else { return };
        match key.code {
            KeyCode::Esc => {
                let selected = self.store.profiles.len();
                self.screen = Screen::Profiles { selected, confirm_delete: false };
            }
            KeyCode::Enter => {
                let name = name.trim().to_string();
                let taken = self.store.profiles.iter().any(|p| p.name.to_lowercase() == name.to_lowercase());
                if name.is_empty() {
                    *error = Some(strings.name_empty);
                } else if taken {
                    *error = Some(strings.name_taken);
                } else {
                    let mut profile = Profile::new(name, self.store.language);
                    profile.settings.theme = self.store.theme;
                    self.store.profiles.push(profile);
                    self.select_profile(self.store.profiles.len() - 1);
                }
            }
            KeyCode::Backspace => {
                name.pop();
                *error = None;
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) && !c.is_control() => {
                if name.chars().count() < MAX_NAME {
                    name.push(c);
                }
                *error = None;
            }
            _ => {}
        }
    }

    fn menu_key(&mut self, key: KeyEvent, selected: usize) {
        let selected = match key.code {
            KeyCode::Up | KeyCode::Char('k') => selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => (selected + 1).min(MENU_ITEMS - 1),
            KeyCode::Enter => return self.menu_action(selected),
            KeyCode::Char(c @ '1'..='7') => return self.menu_action(c as usize - '1' as usize),
            KeyCode::Esc => return self.open_profiles(),
            KeyCode::Char('q') => {
                self.quit = true;
                return;
            }
            _ => return,
        };
        self.screen = Screen::Menu { selected };
    }

    fn menu_action(&mut self, item: usize) {
        match item {
            0 => {
                let selected = self
                    .current()
                    .and_then(|p| (0..LESSONS.len()).find(|&i| p.lesson_stars(i) == 0))
                    .unwrap_or(0);
                self.screen = Screen::Lessons { selected };
            }
            1 => self.start_speed(),
            2 => self.start_rain(),
            3 => self.screen = Screen::Stats,
            4 => self.screen = Screen::Settings { selected: 0 },
            5 => self.open_profiles(),
            _ => self.quit = true,
        }
    }

    fn lessons_key(&mut self, key: KeyEvent, selected: usize) {
        let last = LESSONS.len() - 1;
        let selected = match key.code {
            KeyCode::Up | KeyCode::Char('k') => selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => (selected + 1).min(last),
            KeyCode::Left | KeyCode::Char('h') => selected.checked_sub(LESSON_ROWS).unwrap_or(selected),
            KeyCode::Right | KeyCode::Char('l') => Some(selected + LESSON_ROWS).filter(|&i| i <= last).unwrap_or(selected),
            KeyCode::Enter => {
                if self.current().is_some_and(|p| p.is_unlocked(selected)) {
                    return self.start_lesson(selected);
                }
                self.notice = Some(self.strings().finish_first.replace("{n}", &selected.to_string()));
                selected
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::Menu { selected: 0 };
                return;
            }
            _ => return,
        };
        self.screen = Screen::Lessons { selected };
    }

    fn start_lesson(&mut self, index: usize) {
        let settings = self.settings();
        let words = words::list(settings.language, settings.kids);
        let text = lessons::generate(index, &words, lessons::text_length(settings.kids), &mut self.rng);
        let session = Session::new(&text, None, settings.relaxed_accents);
        self.screen = Screen::Typing(Typing { mode: Mode::Lesson(index), session, cheer: None });
    }

    fn start_speed(&mut self) {
        let settings = self.settings();
        let text = words::random_text(&words::list(settings.language, settings.kids), 80, &mut self.rng);
        let limit = Duration::from_secs(settings.test_seconds);
        let session = Session::new(&text, Some(limit), settings.relaxed_accents);
        self.screen = Screen::Typing(Typing { mode: Mode::Speed(settings.test_seconds), session, cheer: None });
    }

    fn start_rain(&mut self) {
        let settings = self.settings();
        let Some(profile) = self.current() else { return };
        let keys = unlocked_keys(profile.passed_lessons().max().unwrap_or(0));
        let pool = rain::pool(&words::list(settings.language, settings.kids), &keys, &mut self.rng);
        let game = Rain::new(pool, settings.relaxed_accents);
        self.screen = Screen::Rain(RainPlay { game, over: None, last: Instant::now() });
    }

    fn rain_key(&mut self, key: KeyEvent) {
        let Screen::Rain(play) = &mut self.screen else { return };
        if let Some(over) = &play.over {
            if over.age() < RESULTS_GRACE {
                return;
            }
            match key.code {
                KeyCode::Enter | KeyCode::Tab | KeyCode::Char(' ') => self.start_rain(),
                KeyCode::Esc => self.screen = Screen::Menu { selected: 2 },
                _ => {}
            }
            return;
        }
        let game = &mut play.game;
        match key.code {
            KeyCode::Esc => self.screen = Screen::Menu { selected: 2 },
            KeyCode::Tab => self.start_rain(),
            KeyCode::Enter | KeyCode::Char(' ') if !game.is_started() => game.start(),
            KeyCode::Backspace => game.backspace(),
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => game.type_char(c),
            _ => {}
        }
    }

    fn finish_rain(&mut self) {
        let Screen::Rain(play) = &self.screen else { return };
        let score = play.game.score;
        let personal_best = self.current_mut().is_some_and(|profile| profile.record_rain(score));
        self.save();
        let confetti = personal_best.then(|| Confetti::new(&mut self.rng));
        if let Screen::Rain(play) = &mut self.screen {
            play.over = Some(GameOver { personal_best, confetti, at: Instant::now() });
        }
    }

    fn restart(&mut self, mode: Mode) {
        match mode {
            Mode::Lesson(index) => self.start_lesson(index),
            Mode::Speed(_) => self.start_speed(),
        }
    }

    fn typing_key(&mut self, key: KeyEvent) {
        let Screen::Typing(typing) = &mut self.screen else { return };
        let mode = typing.mode;
        let session = &mut typing.session;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Esc => {
                self.screen = match mode {
                    Mode::Lesson(index) => Screen::Lessons { selected: index },
                    Mode::Speed(_) => Screen::Menu { selected: 1 },
                };
            }
            KeyCode::Tab => self.restart(mode),
            KeyCode::Left | KeyCode::Right if matches!(mode, Mode::Speed(_)) && !session.is_started() => {
                self.cycle_test_length(if key.code == KeyCode::Left { -1 } else { 1 });
                self.start_speed();
            }
            KeyCode::Backspace if ctrl || key.modifiers.contains(KeyModifiers::ALT) => session.delete_word(),
            KeyCode::Backspace => session.backspace(),
            KeyCode::Char('w') if ctrl => session.delete_word(),
            KeyCode::Char(c) if !ctrl => {
                let now = Instant::now();
                session.type_char(c, now);
                if celebrate::cheer(session.streak()).is_some() {
                    typing.cheer = Some((session.streak(), now));
                }
            }
            _ => {}
        }
    }

    fn tick(&mut self, now: Instant) {
        if let Screen::Rain(play) = &mut self.screen {
            // Cap the step so a stalled frame doesn't drop words all at once.
            let seconds = now.saturating_duration_since(play.last).as_secs_f64().min(0.25);
            play.last = now;
            if play.over.is_none() {
                play.game.tick(seconds, &mut self.rng);
                if play.game.is_over() {
                    self.finish_rain();
                }
            }
            return;
        }
        let settings = self.settings();
        let Screen::Typing(typing) = &mut self.screen else { return };
        typing.session.tick(now);
        if matches!(typing.mode, Mode::Speed(_)) && typing.session.remaining() < 80 {
            let more = words::random_text(&words::list(settings.language, settings.kids), 40, &mut self.rng);
            typing.session.extend(&more);
        }
        if typing.session.is_finished() {
            self.finish();
        }
    }

    fn finish(&mut self) {
        let Screen::Typing(Typing { mode, session, .. }) = std::mem::replace(&mut self.screen, Screen::Stats) else {
            return;
        };
        let now = Instant::now();
        let summary = session.summary(now);
        let mut missed: Vec<(char, u32)> = session
            .key_stats()
            .iter()
            .filter(|(_, stat)| stat.misses > 0)
            .map(|(&key, stat)| (key, stat.misses))
            .collect();
        missed.sort_by(|a, b| b.1.cmp(&a.1));

        let stars = match mode {
            Mode::Lesson(_) => lessons::stars(summary.accuracy, self.settings().kids),
            Mode::Speed(_) => 0,
        };
        let personal_best = match self.current_mut() {
            Some(profile) => {
                profile.merge_keys(session.key_stats());
                match mode {
                    Mode::Lesson(index) => profile.record_lesson(index, stars, summary.wpm, summary.accuracy),
                    Mode::Speed(seconds) => profile.record_test(TestRecord {
                        wpm: summary.wpm,
                        accuracy: summary.accuracy,
                        seconds,
                        language: profile.settings.language,
                        timestamp: storage::now(),
                    }),
                }
            }
            None => false,
        };
        self.save();
        let confetti = (stars > 0 || personal_best).then(|| Confetti::new(&mut self.rng));
        self.screen = Screen::Results(Results {
            mode,
            wpm: summary.wpm,
            raw_wpm: summary.raw_wpm,
            accuracy: summary.accuracy,
            correct: summary.correct,
            errors: summary.errors,
            seconds: summary.seconds,
            samples: session.samples().to_vec(),
            stars,
            personal_best,
            missed: missed.into_iter().take(5).map(|(key, _)| key).collect(),
            best_streak: session.best_streak(),
            confetti,
            shown_at: now,
        });
    }

    fn results_key(&mut self, key: KeyEvent) {
        let Screen::Results(results) = &self.screen else { return };
        if results.shown_at.elapsed() < RESULTS_GRACE {
            return;
        }
        let passed = results.stars > 0;
        match (results.mode, key.code) {
            (Mode::Lesson(index), KeyCode::Enter) if passed && index + 1 < LESSONS.len() => self.start_lesson(index + 1),
            // After the last lesson, move on to speed tests rather than repeating it.
            (Mode::Lesson(_), KeyCode::Enter) if passed => self.start_speed(),
            (Mode::Lesson(index), KeyCode::Enter | KeyCode::Tab) => self.start_lesson(index),
            (Mode::Lesson(index), KeyCode::Esc) => {
                let selected = if passed { (index + 1).min(LESSONS.len() - 1) } else { index };
                self.screen = Screen::Lessons { selected };
            }
            (Mode::Speed(_), KeyCode::Enter | KeyCode::Tab) => self.start_speed(),
            (Mode::Speed(_), KeyCode::Esc) => self.screen = Screen::Menu { selected: 1 },
            _ => {}
        }
    }

    fn stats_key(&mut self, key: KeyEvent) {
        if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
            self.screen = Screen::Menu { selected: 3 };
        }
    }

    fn settings_key(&mut self, key: KeyEvent, selected: usize) {
        let selected = match key.code {
            KeyCode::Up | KeyCode::Char('k') => selected.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => (selected + 1).min(SETTINGS_ITEMS - 1),
            KeyCode::Left | KeyCode::Char('h') => {
                self.change_setting(selected, -1);
                selected
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
                self.change_setting(selected, 1);
                selected
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.screen = Screen::Menu { selected: 4 };
                return;
            }
            _ => return,
        };
        self.screen = Screen::Settings { selected };
    }

    fn change_setting(&mut self, item: usize, step: isize) {
        if item == 3 {
            return self.cycle_test_length(step);
        }
        let Some(profile) = self.current_mut() else { return };
        let settings = &mut profile.settings;
        match item {
            0 => settings.kids = !settings.kids,
            1 => settings.language = settings.language.next(),
            2 => settings.relaxed_accents = !settings.relaxed_accents,
            4 => settings.keyboard_in_test = !settings.keyboard_in_test,
            _ => settings.theme = settings.theme.next(),
        }
        let (language, theme) = (settings.language, settings.theme);
        self.store.language = language;
        self.store.theme = theme;
        self.save();
    }

    fn cycle_test_length(&mut self, step: isize) {
        let Some(profile) = self.current_mut() else { return };
        let settings = &mut profile.settings;
        let current = TEST_LENGTHS.iter().position(|&s| s == settings.test_seconds).unwrap_or(1);
        let next = (current as isize + step).rem_euclid(TEST_LENGTHS.len() as isize) as usize;
        settings.test_seconds = TEST_LENGTHS[next];
        self.save();
    }
}
