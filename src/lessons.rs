//! The lesson path: letters two at a time from the home row, then Shift,
//! punctuation, numbers and symbols.

use crate::keyboard::SHIFT;
use crate::rng::Rng;

pub const LESSONS: [&[char]; 23] = [
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
    &[SHIFT],
    &['/', '?'],
    &['\'', '"', ':'],
    &['1', '2', '3', '4', '5'],
    &['6', '7', '8', '9', '0'],
    &['-', '!', '(', ')'],
    &['@', '#', '$', '%'],
    &['&', '*', '+', '='],
];

pub const MAX_STARS: u32 = 3 * LESSONS.len() as u32;

/// The accuracy needed to earn a star and unlock the next lesson.
pub const PASS_ACCURACY: f64 = 85.0;

const TEXT_LENGTH: usize = 120;

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

/// Keys practiced in drills and words: the letters, plus `;` on the home row.
fn is_letter(c: char) -> bool {
    c.is_alphabetic() || c == ';'
}

/// Punctuation and symbols, which get added to words rather than drilled.
fn is_mark(c: char) -> bool {
    !is_letter(c) && !c.is_ascii_digit() && c != SHIFT
}

/// Builds practice text for a lesson: a warm-up on the new keys, drills mixing
/// them with the keys learned so far, then real words once enough of them can
/// be typed with what's been learned. Numbers, marks and capitals join in once
/// they've been taught.
pub fn generate(index: usize, words: &[&str], rng: &mut Rng) -> String {
    let taught = unlocked_keys(index);
    let new = LESSONS[index];
    let only = |keys: &[char], kind: fn(char) -> bool| -> Vec<char> { keys.iter().copied().filter(|&c| kind(c)).collect() };
    let (letters, new_letters) = (only(&taught, is_letter), only(new, is_letter));
    let (digits, new_digits) = (only(&taught, |c| c.is_ascii_digit()), only(new, |c| c.is_ascii_digit()));
    let (marks, new_marks) = (only(&taught, is_mark), only(new, is_mark));
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
    for new_keys in [&new_letters, &new_digits] {
        if !new_keys.is_empty() {
            for _ in 0..3 {
                tokens.push(drill(new_keys, new_keys, rng));
            }
        }
    }
    if !new_letters.is_empty() {
        for _ in 0..4 {
            tokens.push(drill(&letters, focus, rng));
        }
    }

    let mut repeats = 0;
    while length(&tokens) < TEXT_LENGTH {
        let token = if !digits.is_empty() && rng.chance(if new_digits.is_empty() { 6 } else { 30 }) {
            let pool = if !new_digits.is_empty() && rng.chance(60) { &new_digits } else { &digits };
            number(pool, rng)
        } else if known.len() >= 6 {
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
        if !new_marks.is_empty() && rng.chance(35) {
            add_mark(token, *rng.pick(&new_marks), &known, rng);
        } else if !marks.is_empty() && rng.chance(12) {
            add_mark(token, *rng.pick(&marks), &known, rng);
        }
    }

    // Chance can skip a key, so make sure every new one shows up.
    for &key in new.iter().filter(|&&key| key != SHIFT) {
        if tokens.iter().any(|token| token.contains(key)) {
            continue;
        }
        let plain: Vec<usize> = (0..tokens.len()).filter(|&i| tokens[i].chars().all(char::is_alphanumeric)).collect();
        let i = if plain.is_empty() { rng.below(tokens.len()) } else { *rng.pick(&plain) };
        match key {
            _ if key.is_ascii_digit() => tokens[i] = drill(&digits, &[key], rng),
            _ if is_letter(key) => tokens[i] = drill(&letters, &[key], rng),
            _ => add_mark(&mut tokens[i], key, &known, rng),
        }
    }

    if taught.contains(&SHIFT) {
        let chance = if new.contains(&SHIFT) { 40 } else { 10 };
        let mut sentence_start = true;
        for token in &mut tokens {
            if sentence_start || rng.chance(chance) {
                capitalize(token);
            }
            sentence_start = token.ends_with(['.', '?', '!']);
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

/// One to four of `digits`, without a leading zero.
fn number(digits: &[char], rng: &mut Rng) -> String {
    let leading: Vec<char> = digits.iter().copied().filter(|&d| d != '0').collect();
    let mut text = String::from(*rng.pick(if leading.is_empty() { digits } else { &leading[..] }));
    for _ in 1..rng.range(1, 4) {
        text.push(*rng.pick(digits));
    }
    text
}

/// Puts `mark` into `token` the way it shows up in real text. Marks that need
/// numbers are only taught after every digit.
fn add_mark(token: &mut String, mark: char, words: &[&str], rng: &mut Rng) {
    *token = match mark {
        '"' => format!("\"{token}\""),
        '(' | ')' => format!("({token})"),
        '\'' => format!("'{token}'"),
        '/' | '-' | '&' => {
            let other = if words.is_empty() { token.clone() } else { rng.pick(words).to_string() };
            if mark == '&' { format!("{token} & {other}") } else { format!("{token}{mark}{other}") }
        }
        '@' => format!("@{token}"),
        '#' | '$' => format!("{mark}{}", rng.range(1, 500)),
        '%' => format!("{}%", rng.range(1, 100)),
        '*' | '+' => format!("{}{mark}{}", rng.range(2, 12), rng.range(2, 12)),
        '=' => {
            let (a, b) = (rng.range(1, 50), rng.range(1, 50));
            format!("{a}+{b}={}", a + b)
        }
        _ => format!("{token}{mark}"),
    };
}

/// Uppercases the first letter, skipping marks such as an opening quote.
fn capitalize(token: &mut String) {
    if let Some((i, c)) = token.char_indices().find(|(_, c)| c.is_alphabetic()) {
        token.replace_range(i..i + c.len_utf8(), &c.to_uppercase().to_string());
    }
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
                let shift = allowed.contains(&SHIFT);
                for seed in 0..20 {
                    let text = generate(index, &words, &mut Rng::new(seed));
                    assert!(text.chars().count() >= TEXT_LENGTH - 10, "lesson {index}: {text}");
                    for c in text.chars().filter(|&c| c != ' ') {
                        let key = if shift { c.to_lowercase().next().unwrap_or(c) } else { c };
                        assert!(allowed.contains(&key), "lesson {index} used {c:?}: {text}");
                    }
                }
            }
        }
    }

    #[test]
    fn every_lesson_practices_its_new_keys() {
        for language in Language::ALL {
            let words = words::list(language);
            for (index, keys) in LESSONS.iter().enumerate() {
                for seed in 0..20 {
                    let text = generate(index, &words, &mut Rng::new(seed));
                    for &key in *keys {
                        let used = if key == SHIFT { text.chars().any(char::is_uppercase) } else { text.contains(key) };
                        assert!(used, "lesson {index} never uses {key:?}: {text}");
                    }
                }
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
