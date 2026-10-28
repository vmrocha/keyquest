//! Word lists for lessons and speed tests, one word per line.

use crate::i18n::Language;
use crate::rng::Rng;

const ENGLISH: &str = include_str!("../assets/words/en.txt");
const PORTUGUESE: &str = include_str!("../assets/words/pt-br.txt");

pub fn list(language: Language) -> Vec<&'static str> {
    let raw = match language {
        Language::English => ENGLISH,
        Language::Portuguese => PORTUGUESE,
    };
    raw.lines().map(str::trim).filter(|word| !word.is_empty()).collect()
}

/// `count` random words separated by spaces, never the same word twice in a row.
pub fn random_text(words: &[&str], count: usize, rng: &mut Rng) -> String {
    let mut picked: Vec<&str> = Vec::with_capacity(count);
    while picked.len() < count {
        let word = *rng.pick(words);
        if picked.last() != Some(&word) {
            picked.push(word);
        }
    }
    picked.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_hold_lowercase_single_words() {
        for language in Language::ALL {
            let words = list(language);
            assert!(words.len() > 200, "{language:?} has only {} words", words.len());
            for word in words {
                assert!(word.chars().all(char::is_lowercase), "{word:?}");
            }
        }
    }

    #[test]
    fn random_text_has_the_requested_word_count() {
        let words = list(Language::Portuguese);
        let text = random_text(&words, 25, &mut Rng::new(1));
        assert_eq!(text.split(' ').count(), 25);
    }
}
