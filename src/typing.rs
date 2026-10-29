//! The typing engine: compares keystrokes with a target text and keeps score.
//!
//! Every keystroke moves the cursor forward, right or wrong. Mistakes stay
//! visible until fixed with backspace, but accuracy counts every wrong
//! keystroke, even ones fixed later.

use std::collections::BTreeMap;
use std::ops::Range;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::keyboard::{base_key, without_accent};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharState {
    Correct,
    Wrong,
    Cursor,
    Pending,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyStat {
    pub total: u32,
    pub misses: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Summary {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub correct: usize,
    pub errors: u32,
    pub seconds: f64,
}

pub struct Session {
    target: Vec<char>,
    input: Vec<char>,
    limit: Option<Duration>,
    relaxed_accents: bool,
    started: Option<Instant>,
    ended: Option<Instant>,
    keystrokes: u32,
    errors: u32,
    streak: u32,
    best_streak: u32,
    key_stats: BTreeMap<char, KeyStat>,
    samples: Vec<(f64, f64)>,
}

impl Session {
    /// With a `limit` the session ends when time runs out, otherwise when the
    /// last character is typed.
    pub fn new(text: &str, limit: Option<Duration>, relaxed_accents: bool) -> Self {
        Self {
            target: text.chars().collect(),
            input: Vec::new(),
            limit,
            relaxed_accents,
            started: None,
            ended: None,
            keystrokes: 0,
            errors: 0,
            streak: 0,
            best_streak: 0,
            key_stats: BTreeMap::new(),
            samples: Vec::new(),
        }
    }

    /// Appends more words, for timed sessions that would otherwise run out of text.
    pub fn extend(&mut self, text: &str) {
        if !self.target.is_empty() {
            self.target.push(' ');
        }
        self.target.extend(text.chars());
    }

    pub fn type_char(&mut self, c: char, now: Instant) {
        if self.is_finished() || self.input.len() >= self.target.len() {
            return;
        }
        self.started.get_or_insert(now);
        let expected = self.target[self.input.len()];
        let correct = self.matches(c, expected);
        let stat = self.key_stats.entry(base_key(expected)).or_default();
        stat.total += 1;
        self.keystrokes += 1;
        if correct {
            self.streak += 1;
            self.best_streak = self.best_streak.max(self.streak);
        } else {
            stat.misses += 1;
            self.errors += 1;
            self.streak = 0;
        }
        self.input.push(c);
        if self.limit.is_none() && self.input.len() == self.target.len() {
            self.end(now);
        }
    }

    pub fn backspace(&mut self) {
        if !self.is_finished() {
            self.input.pop();
        }
    }

    /// Deletes back to the start of the current word.
    pub fn delete_word(&mut self) {
        if self.is_finished() {
            return;
        }
        while self.input.last() == Some(&' ') {
            self.input.pop();
        }
        while self.input.last().is_some_and(|&c| c != ' ') {
            self.input.pop();
        }
    }

    /// Ends timed sessions and records the per-second speed samples.
    pub fn tick(&mut self, now: Instant) {
        let Some(started) = self.started else { return };
        if self.is_finished() {
            return;
        }
        if let Some(limit) = self.limit
            && now.duration_since(started) >= limit
        {
            self.end(started + limit);
            return;
        }
        self.sample(now);
    }

    fn end(&mut self, at: Instant) {
        self.sample(at);
        self.ended = Some(at);
    }

    fn sample(&mut self, now: Instant) {
        let seconds = self.elapsed(now).as_secs_f64();
        let wpm = self.summary(now).wpm;
        while self.samples.len() as f64 + 1.0 <= seconds {
            let second = self.samples.len() as f64 + 1.0;
            self.samples.push((second, wpm));
        }
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        let Some(started) = self.started else {
            return Duration::ZERO;
        };
        let elapsed = self.ended.unwrap_or(now).saturating_duration_since(started);
        self.limit.map_or(elapsed, |limit| elapsed.min(limit))
    }

    pub fn summary(&self, now: Instant) -> Summary {
        let seconds = self.elapsed(now).as_secs_f64();
        // Below one second the numbers swing wildly, so treat it as one second.
        let minutes = seconds.max(1.0) / 60.0;
        let correct = self
            .input
            .iter()
            .zip(&self.target)
            .filter(|&(&typed, &expected)| self.matches(typed, expected))
            .count();
        let accuracy = if self.keystrokes == 0 {
            100.0
        } else {
            100.0 * f64::from(self.keystrokes - self.errors) / f64::from(self.keystrokes)
        };
        Summary {
            wpm: correct as f64 / 5.0 / minutes,
            raw_wpm: f64::from(self.keystrokes) / 5.0 / minutes,
            accuracy,
            correct,
            errors: self.errors,
            seconds,
        }
    }

    pub fn state(&self, index: usize) -> CharState {
        match self.input.get(index) {
            Some(&typed) if self.matches(typed, self.target[index]) => CharState::Correct,
            Some(_) => CharState::Wrong,
            None if index == self.input.len() && !self.is_finished() => CharState::Cursor,
            None => CharState::Pending,
        }
    }

    pub fn target(&self) -> &[char] {
        &self.target
    }

    pub fn position(&self) -> usize {
        self.input.len()
    }

    /// The character the typist should press next.
    pub fn expected(&self) -> Option<char> {
        if self.is_finished() {
            return None;
        }
        self.target.get(self.input.len()).copied()
    }

    pub fn remaining(&self) -> usize {
        self.target.len() - self.input.len()
    }

    pub fn progress(&self) -> f64 {
        if self.target.is_empty() {
            return 0.0;
        }
        self.input.len() as f64 / self.target.len() as f64
    }

    pub fn is_started(&self) -> bool {
        self.started.is_some()
    }

    pub fn is_finished(&self) -> bool {
        self.ended.is_some()
    }

    pub fn limit(&self) -> Option<Duration> {
        self.limit
    }

    /// Correct keystrokes in a row since the last mistake.
    pub fn streak(&self) -> u32 {
        self.streak
    }

    pub fn best_streak(&self) -> u32 {
        self.best_streak
    }

    pub fn key_stats(&self) -> &BTreeMap<char, KeyStat> {
        &self.key_stats
    }

    pub fn samples(&self) -> &[(f64, f64)] {
        &self.samples
    }

    fn matches(&self, typed: char, expected: char) -> bool {
        typed == expected || (self.relaxed_accents && without_accent(typed) == without_accent(expected))
    }
}

/// Splits text into lines of at most `width` characters, breaking after spaces.
/// Words longer than a line are cut.
pub fn wrap(text: &[char], width: usize) -> Vec<Range<usize>> {
    let width = width.max(1);
    let mut lines = Vec::new();
    let mut start = 0;
    while start < text.len() {
        if text.len() - start <= width {
            lines.push(start..text.len());
            break;
        }
        let end = match text[start..start + width].iter().rposition(|&c| c == ' ') {
            Some(space) => start + space + 1,
            None => start + width,
        };
        lines.push(start..end);
        start = end;
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_all(session: &mut Session, text: &str, now: Instant) {
        for c in text.chars() {
            session.type_char(c, now);
        }
    }

    #[test]
    fn typing_the_whole_text_finishes_an_untimed_session() {
        let now = Instant::now();
        let mut session = Session::new("fj jf", None, false);
        type_all(&mut session, "fj jf", now);
        assert!(session.is_finished());
        let summary = session.summary(now);
        assert_eq!(summary.correct, 5);
        assert_eq!(summary.accuracy, 100.0);
    }

    #[test]
    fn fixed_mistakes_still_count_against_accuracy() {
        let now = Instant::now();
        let mut session = Session::new("ab", None, false);
        session.type_char('x', now);
        assert_eq!(session.state(0), CharState::Wrong);
        session.backspace();
        assert_eq!(session.state(0), CharState::Cursor);
        type_all(&mut session, "ab", now);
        let summary = session.summary(now);
        assert_eq!(summary.correct, 2);
        assert_eq!(summary.errors, 1);
        assert!((summary.accuracy - 200.0 / 3.0).abs() < 1e-9);
        assert_eq!(session.key_stats()[&'a'], KeyStat { total: 2, misses: 1 });
    }

    #[test]
    fn mistakes_end_a_streak() {
        let now = Instant::now();
        let mut session = Session::new("abcdef", None, false);
        type_all(&mut session, "abc", now);
        assert_eq!(session.streak(), 3);
        session.type_char('x', now);
        assert_eq!(session.streak(), 0);
        session.backspace();
        type_all(&mut session, "de", now);
        assert_eq!(session.streak(), 2);
        assert_eq!(session.best_streak(), 3);
    }

    #[test]
    fn relaxed_accents_accept_the_plain_letter() {
        let now = Instant::now();
        let mut strict = Session::new("é", None, false);
        strict.type_char('e', now);
        assert_eq!(strict.summary(now).correct, 0);

        let mut relaxed = Session::new("é", None, true);
        relaxed.type_char('e', now);
        assert_eq!(relaxed.summary(now).correct, 1);

        // Relaxed is only about accents: Shift still counts.
        let mut shifted = Session::new("Fé?", None, true);
        type_all(&mut shifted, "fe/", now);
        assert_eq!(shifted.summary(now).correct, 1);
        assert_eq!(shifted.key_stats()[&'/'], KeyStat { total: 1, misses: 1 });
    }

    #[test]
    fn timed_sessions_end_when_time_runs_out() {
        let start = Instant::now();
        let mut session = Session::new("one two three", Some(Duration::from_secs(2)), false);
        type_all(&mut session, "one", start);
        session.tick(start + Duration::from_secs(1));
        assert!(!session.is_finished());
        session.tick(start + Duration::from_secs(3));
        assert!(session.is_finished());
        assert_eq!(session.elapsed(start + Duration::from_secs(9)), Duration::from_secs(2));
        assert_eq!(session.samples().len(), 2);
    }

    #[test]
    fn delete_word_goes_back_to_the_previous_space() {
        let now = Instant::now();
        let mut session = Session::new("hello world", None, false);
        type_all(&mut session, "hello wo", now);
        session.delete_word();
        assert_eq!(session.position(), 6);
        session.delete_word();
        assert_eq!(session.position(), 0);
    }

    #[test]
    fn wrap_breaks_after_spaces() {
        let text: Vec<char> = "the quick brown fox".chars().collect();
        assert_eq!(wrap(&text, 10), vec![0..10, 10..19]);
        assert_eq!(wrap(&text, 8), vec![0..4, 4..10, 10..16, 16..19]);
        let long: Vec<char> = "abcdefgh".chars().collect();
        assert_eq!(wrap(&long, 3), vec![0..3, 3..6, 6..8]);
        assert!(wrap(&[], 10).is_empty());
    }
}
