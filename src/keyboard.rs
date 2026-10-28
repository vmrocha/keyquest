//! The physical keyboard: which keys exist, and which finger presses each one.

/// US QWERTY letter rows, top to bottom.
pub const ROWS: [&str; 3] = ["qwertyuiop", "asdfghjkl;", "zxcvbnm,./"];

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

pub fn finger(key: char) -> Finger {
    match key {
        'q' | 'a' | 'z' => Finger::LeftPinky,
        'w' | 's' | 'x' => Finger::LeftRing,
        'e' | 'd' | 'c' => Finger::LeftMiddle,
        'r' | 't' | 'f' | 'g' | 'v' | 'b' => Finger::LeftIndex,
        'y' | 'u' | 'h' | 'j' | 'n' | 'm' => Finger::RightIndex,
        'i' | 'k' | ',' => Finger::RightMiddle,
        'o' | 'l' | '.' => Finger::RightRing,
        'p' | ';' | '/' => Finger::RightPinky,
        _ => Finger::Thumb,
    }
}

/// The key a character lives on: lowercase, with accents removed.
pub fn base_key(c: char) -> char {
    let lower = c.to_lowercase().next().unwrap_or(c);
    match lower {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accents_map_to_their_base_key() {
        assert_eq!(base_key('ã'), 'a');
        assert_eq!(base_key('É'), 'e');
        assert_eq!(base_key('ç'), 'c');
        assert_eq!(base_key(';'), ';');
    }

    #[test]
    fn every_key_has_a_finger_other_than_the_thumb() {
        for key in ROWS.iter().flat_map(|row| row.chars()) {
            assert_ne!(finger(key), Finger::Thumb, "{key}");
        }
        assert_eq!(finger(' '), Finger::Thumb);
    }
}
