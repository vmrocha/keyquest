//! The lesson path: two new keys at a time, starting from the home row.

use crate::rng::Rng;

pub const LESSONS: [&[char]; 15] = [
    &['f', 'j'],
    &['d', 'k'],
    &['s', 'l'],
    &['a', ';'],
    &['g', 'h'],
    &['e', 'i'],
    &['r', 'u'],
    &['t', 'o'],
    &['n', 'm'],
    &['c', 'v'],
    &['w', 'y'],
    &['b', 'p'],
    &['x', 'q'],
    &['z', ','],
    &['.'],
];

pub const MAX_STARS: u32 = 3 * LESSONS.len() as u32;

/// The accuracy needed to earn a star and unlock the next lesson.
pub const PASS_ACCURACY: f64 = 85.0;

const TEXT_LENGTH: usize = 120;
const PUNCTUATION: [char; 2] = [',', '.'];

pub fn stars(accuracy: f64) -> u8 {
    match accuracy {
        a if a >= 98.0 => 3,
        a if a >= 93.0 => 2,
        a if a >= PASS_ACCURACY => 1,
        _ => 0,
    }
}

/// Every key taught up to and including lesson `index`.
pub fn unlocked_keys(index: usize) -> Vec<char> {
    LESSONS[..=index].iter().flat_map(|keys| keys.iter().copied()).collect()
}

/// Builds practice text for a lesson: a warm-up on the new keys, drills mixing
/// them with the keys learned so far, then real words once enough of them can
/// be typed with what's been learned.
pub fn generate(index: usize, words: &[&str], rng: &mut Rng) -> String {
    let (letters, punctuation): (Vec<char>, Vec<char>) =
        unlocked_keys(index).into_iter().partition(|c| !PUNCTUATION.contains(c));
    let (new_letters, new_punctuation): (Vec<char>, Vec<char>) =
        LESSONS[index].iter().copied().partition(|c| !PUNCTUATION.contains(c));
    let focus = if new_letters.is_empty() { &letters } else { &new_letters };

    let known: Vec<&str> = words
        .iter()
        .copied()
        .filter(|word| word.chars().count() >= 2 && word.chars().all(|c| letters.contains(&c)))
        .collect();
    let fresh: Vec<&str> = known
        .iter()
        .copied()
        .filter(|word| word.chars().any(|c| new_letters.contains(&c)))
        .collect();

    let mut tokens: Vec<String> = Vec::new();
    if !new_letters.is_empty() {
        for _ in 0..3 {
            tokens.push(drill(&new_letters, &new_letters, rng));
        }
    }
    for _ in 0..4 {
        tokens.push(drill(&letters, focus, rng));
    }

    let mut repeats = 0;
    while length(&tokens) < TEXT_LENGTH {
        let token = if known.len() >= 6 {
            let pool = if !fresh.is_empty() && rng.chance(60) { &fresh } else { &known };
            rng.pick(pool).to_string()
        } else {
            drill(&letters, focus, rng)
        };
        if tokens.last() == Some(&token) && repeats < 5 {
            repeats += 1;
            continue;
        }
        repeats = 0;
        tokens.push(token);
    }

    for token in &mut tokens {
        if !new_punctuation.is_empty() && rng.chance(35) {
            token.push(*rng.pick(&new_punctuation));
        } else if !punctuation.is_empty() && rng.chance(12) {
            token.push(*rng.pick(&punctuation));
        }
    }
    tokens.join(" ")
}

/// A short random group of keys that always includes at least one `focus` key.
fn drill(pool: &[char], focus: &[char], rng: &mut Rng) -> String {
    let len = rng.range(2, 5);
    let mut keys: Vec<char> = (0..len).map(|_| *rng.pick(pool)).collect();
    if !keys.iter().any(|c| focus.contains(c)) {
        keys[rng.below(len)] = *rng.pick(focus);
    }
    keys.into_iter().collect()
}

fn length(tokens: &[String]) -> usize {
    tokens.iter().map(|t| t.chars().count() + 1).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;
    use crate::words;

    #[test]
    fn lessons_only_use_keys_taught_so_far() {
        for language in Language::ALL {
            let words = words::list(language);
            for index in 0..LESSONS.len() {
                let allowed = unlocked_keys(index);
                for seed in 0..20 {
                    let text = generate(index, &words, &mut Rng::new(seed));
                    assert!(text.chars().count() >= TEXT_LENGTH - 10, "lesson {index}: {text}");
                    for c in text.chars().filter(|&c| c != ' ') {
                        assert!(allowed.contains(&c), "lesson {index} used {c:?}: {text}");
                    }
                }
            }
        }
    }

    #[test]
    fn every_lesson_practices_its_new_keys() {
        let words = words::list(Language::English);
        for (index, keys) in LESSONS.iter().enumerate() {
            let text = generate(index, &words, &mut Rng::new(7));
            for key in *keys {
                assert!(text.contains(*key), "lesson {index} never uses {key:?}: {text}");
            }
        }
    }

    #[test]
    fn stars_follow_accuracy() {
        assert_eq!(stars(99.0), 3);
        assert_eq!(stars(95.0), 2);
        assert_eq!(stars(85.0), 1);
        assert_eq!(stars(84.9), 0);
    }
}
