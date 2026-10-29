//! The quest map: lessons grouped into worlds along one winding path.
//!
//! Each world is a row of the map. Rows take turns running right and left,
//! and each one starts right under the last lesson of the row above.

use std::ops::Range;

use crate::lessons::LESSONS;
use crate::storage::Profile;

/// The lessons in each world.
pub const WORLDS: [Range<usize>; 6] = [0..5, 5..8, 8..15, 15..18, 18..20, 20..23];

pub fn world_of(lesson: usize) -> usize {
    WORLDS.iter().position(|world| world.contains(&lesson)).unwrap_or(WORLDS.len() - 1)
}

/// Whether the path runs right through `world`, or left.
pub fn runs_right(world: usize) -> bool {
    world.is_multiple_of(2)
}

/// Each lesson's column on the map, counted in lessons from the left.
pub fn columns() -> [usize; LESSONS.len()] {
    let mut columns = [0; LESSONS.len()];
    for (world, lessons) in WORLDS.iter().enumerate() {
        for lesson in lessons.clone().skip(1) {
            columns[lesson] = match runs_right(world) {
                true => columns[lesson - 1] + 1,
                false => columns[lesson - 1] - 1,
            };
        }
        if let Some(next) = WORLDS.get(world + 1) {
            columns[next.start] = columns[lessons.end - 1];
        }
    }
    columns
}

/// How many lessons wide the map is.
pub fn width() -> usize {
    columns().iter().max().map_or(0, |widest| widest + 1)
}

/// Whether every lesson in `world` has a star.
pub fn is_cleared(profile: &Profile, world: usize) -> bool {
    WORLDS[world].clone().all(|lesson| profile.lesson_stars(lesson) > 0)
}

/// The lesson an arrow key leads to from `lesson`: `dx` columns along its
/// world, or the closest lesson `dy` worlds down. Stays put at the edges.
pub fn step(lesson: usize, dx: isize, dy: isize) -> usize {
    let columns = columns();
    let Some(world) = world_of(lesson).checked_add_signed(dy).and_then(|world| WORLDS.get(world)) else {
        return lesson;
    };
    let target = columns[lesson] as isize + dx;
    let distance = |i: &usize| (columns[*i] as isize - target).abs();
    world.clone().min_by_key(distance).filter(|i| dy != 0 || distance(i) == 0).unwrap_or(lesson)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::Language;

    #[test]
    fn worlds_cover_every_lesson_once() {
        let lessons: Vec<usize> = WORLDS.iter().flat_map(|world| world.clone()).collect();
        assert_eq!(lessons, (0..LESSONS.len()).collect::<Vec<_>>());
        assert_eq!(world_of(0), 0);
        assert_eq!(world_of(LESSONS.len() - 1), WORLDS.len() - 1);
    }

    #[test]
    fn the_path_winds_without_gaps() {
        let columns = columns();
        for (world, lessons) in WORLDS.iter().enumerate() {
            for lesson in lessons.clone().skip(1) {
                assert_eq!(columns[lesson].abs_diff(columns[lesson - 1]), 1, "lesson {lesson}");
            }
            // Each world starts under the end of the one before.
            if world > 0 {
                assert_eq!(columns[lessons.start], columns[lessons.start - 1]);
            }
        }
        assert_eq!(columns[0], 0);
        assert_eq!(width(), 9);
    }

    #[test]
    fn arrows_move_around_the_map() {
        // Home Row Meadow is lessons 1 to 5, left to right; Top Row Hills runs back from 6.
        assert_eq!(step(0, 1, 0), 1);
        assert_eq!(step(0, -1, 0), 0);
        assert_eq!(step(4, 1, 0), 4);
        assert_eq!(step(4, 0, 1), 5);
        assert_eq!(step(0, 0, 1), 7);
        assert_eq!(step(5, 1, 0), 5);
        assert_eq!(step(5, -1, 0), 6);
        assert_eq!(step(7, 0, -1), 2);
        assert_eq!(step(2, 0, -1), 2);
        assert_eq!(step(LESSONS.len() - 1, 0, 1), LESSONS.len() - 1);
    }

    #[test]
    fn a_world_is_cleared_when_every_lesson_has_a_star() {
        let mut profile = Profile::new("Lia".into(), Language::English);
        for lesson in 0..4 {
            profile.record_lesson(lesson, 1, 20.0, 90.0);
        }
        assert!(!is_cleared(&profile, 0));
        profile.record_lesson(4, 2, 20.0, 95.0);
        assert!(is_cleared(&profile, 0));
        assert!(!is_cleared(&profile, 1));
    }
}
