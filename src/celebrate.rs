//! Rewards: cheers for long runs of correct keys, and confetti for wins.

use crate::rng::Rng;

/// How long a cheer stays on screen, in seconds.
pub const CHEER_SECONDS: f64 = 1.5;

/// The cheer for reaching `streak` correct keys in a row, when that's a
/// milestone: 0 at 10, 1 at 25, 2 at 50 and 3 at every hundred.
pub fn cheer(streak: u32) -> Option<usize> {
    match streak {
        10 => Some(0),
        25 => Some(1),
        50 => Some(2),
        n if n > 0 && n % 100 == 0 => Some(3),
        _ => None,
    }
}

const PIECES: usize = 70;
const GLYPHS: [char; 8] = ['•', '*', '+', '✦', '★', '◆', '▪', '♦'];

struct Piece {
    x: f64,
    /// Seconds before it starts falling.
    delay: f64,
    /// Screen heights per second.
    speed: f64,
    /// Phase of the side-to-side drift.
    phase: f64,
    glyph: char,
    color: usize,
}

/// A burst of confetti that falls once from the top of the screen.
pub struct Confetti {
    pieces: Vec<Piece>,
}

impl Confetti {
    pub fn new(rng: &mut Rng) -> Self {
        let pieces = (0..PIECES)
            .map(|_| Piece {
                x: rng.below(1000) as f64 / 1000.0,
                delay: rng.below(900) as f64 / 1000.0,
                speed: 0.6 + rng.below(600) as f64 / 1000.0,
                phase: rng.below(628) as f64 / 100.0,
                glyph: *rng.pick(&GLYPHS),
                color: rng.below(9),
            })
            .collect();
        Self { pieces }
    }

    /// The pieces on screen `seconds` after the burst, as (x, y, glyph, color),
    /// with x and y from 0 to 1 and y growing downward. Empty once they've all landed.
    pub fn pieces(&self, seconds: f64) -> impl Iterator<Item = (f64, f64, char, usize)> + '_ {
        self.pieces.iter().filter_map(move |piece| {
            let falling = seconds - piece.delay;
            let y = falling * piece.speed;
            let x = piece.x + 0.015 * (falling * 4.0 + piece.phase).sin();
            (falling >= 0.0 && y < 1.0).then_some((x.clamp(0.0, 1.0), y, piece.glyph, piece.color))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn milestones_get_cheers() {
        assert_eq!(cheer(9), None);
        assert_eq!(cheer(10), Some(0));
        assert_eq!(cheer(25), Some(1));
        assert_eq!(cheer(50), Some(2));
        assert_eq!(cheer(99), None);
        assert_eq!(cheer(100), Some(3));
        assert_eq!(cheer(300), Some(3));
        assert_eq!(cheer(0), None);
    }

    #[test]
    fn confetti_falls_and_lands() {
        let confetti = Confetti::new(&mut Rng::new(7));
        assert_eq!(confetti.pieces(0.0).count(), confetti.pieces.iter().filter(|p| p.delay == 0.0).count());
        assert!(confetti.pieces(0.8).count() > PIECES / 2);
        for (x, y, _, _) in confetti.pieces(1.2) {
            assert!((0.0..=1.0).contains(&x) && (0.0..1.0).contains(&y));
        }
        assert_eq!(confetti.pieces(3.0).count(), 0);
    }
}
