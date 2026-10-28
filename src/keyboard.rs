//! The physical keyboard: which keys exist, and which finger presses each one.

/// US QWERTY rows, top to bottom.
pub const ROWS: [&str; 4] = ["1234567890-=", "qwertyuiop", "asdfghjkl;'", "zxcvbnm,./"];

/// Stands for the Shift key in lesson key lists.
pub const SHIFT: char = '⇧';

/// Symbols typed with Shift, and the key each one is on.
const SHIFTED: [(char, char); 17] = [
    ('!', '1'),
    ('@', '2'),
    ('#', '3'),
    ('$', '4'),
    ('%', '5'),
    ('^', '6'),
    ('&', '7'),
    ('*', '8'),
    ('(', '9'),
    (')', '0'),
    ('_', '-'),
    ('+', '='),
    (':', ';'),
    ('"', '\''),
    ('<', ','),
    ('>', '.'),
    ('?', '/'),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

/// A key on the on-screen keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// A key by what it types without Shift, `' '` for the space bar.
    Char(char),
    Shift(Hand),
}

impl Key {
    pub fn finger(self) -> Finger {
        match self {
            Key::Char(c) => finger(c),
            Key::Shift(Hand::Left) => Finger::LeftPinky,
            Key::Shift(Hand::Right) => Finger::RightPinky,
        }
    }

    /// Whether typing any of `chars` takes this key. [`SHIFT`] stands for both Shift keys.
    pub fn used_by(self, chars: &[char]) -> bool {
        match self {
            Key::Char(key) => chars.iter().any(|&c| base_key(c) == key),
            Key::Shift(_) => chars.contains(&SHIFT),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Finger {
    LeftPinky,
    LeftRing,
    LeftMiddle,
    LeftIndex,
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
    Thumb,
}

impl Finger {
    fn hand(self) -> Option<Hand> {
        match self {
            Finger::LeftPinky | Finger::LeftRing | Finger::LeftMiddle | Finger::LeftIndex => Some(Hand::Left),
            Finger::Thumb => None,
            _ => Some(Hand::Right),
        }
    }
}

pub fn finger(c: char) -> Finger {
    match base_key(c) {
        '1' | 'q' | 'a' | 'z' => Finger::LeftPinky,
        '2' | 'w' | 's' | 'x' => Finger::LeftRing,
        '3' | 'e' | 'd' | 'c' => Finger::LeftMiddle,
        '4' | '5' | 'r' | 't' | 'f' | 'g' | 'v' | 'b' => Finger::LeftIndex,
        '6' | '7' | 'y' | 'u' | 'h' | 'j' | 'n' | 'm' => Finger::RightIndex,
        '8' | 'i' | 'k' | ',' => Finger::RightMiddle,
        '9' | 'o' | 'l' | '.' => Finger::RightRing,
        '0' | '-' | '=' | 'p' | ';' | '\'' | '/' => Finger::RightPinky,
        _ => Finger::Thumb,
    }
}

/// The key a character lives on: lowercase, with accents and Shift removed.
pub fn base_key(c: char) -> char {
    let plain = without_accent(c).to_lowercase().next().unwrap_or(c);
    SHIFTED.iter().find(|&&(symbol, _)| symbol == plain).map_or(plain, |&(_, key)| key)
}

/// The letter without its accent, keeping its case.
pub fn without_accent(c: char) -> char {
    let plain = match c.to_lowercase().next().unwrap_or(c) {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        _ => return c,
    };
    if c.is_uppercase() { plain.to_ascii_uppercase() } else { plain }
}

/// The symbol Shift types on `key`, for keycap labels.
pub fn shifted(key: char) -> Option<char> {
    SHIFTED.iter().find(|&&(_, k)| k == key).map(|&(symbol, _)| symbol)
}

/// The Shift key to hold for `c`, if any: the one on the other hand, so one
/// hand holds Shift while the other presses the key.
pub fn shift_for(c: char) -> Option<Hand> {
    if !c.is_uppercase() && shifted(base_key(c)) != Some(c) {
        return None;
    }
    match finger(c).hand()? {
        Hand::Left => Some(Hand::Right),
        Hand::Right => Some(Hand::Left),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accents_and_shift_map_to_their_base_key() {
        assert_eq!(base_key('ã'), 'a');
        assert_eq!(base_key('É'), 'e');
        assert_eq!(base_key('ç'), 'c');
        assert_eq!(base_key(';'), ';');
        assert_eq!(base_key('?'), '/');
        assert_eq!(base_key('!'), '1');
        assert_eq!(base_key('"'), '\'');
    }

    #[test]
    fn removing_accents_keeps_the_case() {
        assert_eq!(without_accent('ã'), 'a');
        assert_eq!(without_accent('É'), 'E');
        assert_eq!(without_accent('F'), 'F');
        assert_eq!(without_accent('?'), '?');
    }

    #[test]
    fn shift_comes_from_the_other_hand() {
        assert_eq!(shift_for('F'), Some(Hand::Right));
        assert_eq!(shift_for('J'), Some(Hand::Left));
        assert_eq!(shift_for('?'), Some(Hand::Left));
        assert_eq!(shift_for('@'), Some(Hand::Right));
        assert_eq!(shift_for('f'), None);
        assert_eq!(shift_for('/'), None);
        assert_eq!(shift_for(' '), None);
    }

    #[test]
    fn every_key_has_a_finger_other_than_the_thumb() {
        for key in ROWS.iter().flat_map(|row| row.chars()) {
            assert_ne!(finger(key), Finger::Thumb, "{key}");
            if let Some(symbol) = shifted(key) {
                assert_eq!(finger(symbol), finger(key), "{symbol}");
            }
        }
        assert_eq!(finger(' '), Finger::Thumb);
    }

    #[test]
    fn keys_know_which_characters_use_them() {
        assert!(Key::Char('/').used_by(&['?']));
        assert!(Key::Char('f').used_by(&['F']));
        assert!(!Key::Char('f').used_by(&['j']));
        assert!(Key::Shift(Hand::Left).used_by(&['a', SHIFT]));
        assert!(!Key::Shift(Hand::Right).used_by(&['a']));
    }
}
