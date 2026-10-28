//! The on-screen keyboard.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::keyboard::{Hand, Key, ROWS, shifted};

const KEY_WIDTH: usize = 5;
/// How far each row is shifted right, like a real keyboard's stagger.
const ROW_OFFSETS: [usize; 4] = [0, 3, 4, 6];
/// The number row sets the width, and the right Shift ends under its last key.
const WIDTH: usize = 12 * (KEY_WIDTH + 1) - 1;
/// With a blank line between rows.
pub const HEIGHT: u16 = 9;
/// Without the blank lines, for short windows.
pub const COMPACT_HEIGHT: u16 = 5;

/// Draws the keyboard centered in `area`, compact when it's shorter than
/// [`HEIGHT`]. `style` picks each key's look.
pub fn render(frame: &mut Frame, area: Rect, style: impl Fn(Key) -> Style) {
    let cap = |label: String, width: usize, key: Key| Span::styled(format!("{label:^width$}"), style(key));
    let mut rows = Vec::new();
    for (row, keys) in ROWS.iter().enumerate() {
        let bottom = row == ROWS.len() - 1;
        let offset = ROW_OFFSETS[row];
        let mut spans = if bottom {
            vec![cap("⇧".into(), offset - 1, Key::Shift(Hand::Left)), Span::raw(" ")]
        } else {
            vec![Span::raw(" ".repeat(offset))]
        };
        for (i, key) in keys.chars().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            // Show what Shift types too, the way keycaps do.
            let label = match shifted(key) {
                Some(symbol) => format!("{key} {symbol}"),
                None => key.to_uppercase().to_string(),
            };
            spans.push(cap(label, KEY_WIDTH, Key::Char(key)));
        }
        if bottom {
            let used = offset + keys.chars().count() * (KEY_WIDTH + 1);
            spans.push(Span::raw(" "));
            spans.push(cap("⇧".into(), WIDTH - used, Key::Shift(Hand::Right)));
        }
        rows.push(Line::from(spans));
    }
    // The space bar runs under C to M.
    let start = ROW_OFFSETS[3] + 2 * (KEY_WIDTH + 1);
    let width = 5 * (KEY_WIDTH + 1) - 1;
    rows.push(Line::from(vec![Span::raw(" ".repeat(start)), cap(String::new(), width, Key::Char(' '))]));

    let lines = if area.height >= HEIGHT {
        rows.into_iter().flat_map(|row| [Line::default(), row]).skip(1).collect()
    } else {
        rows
    };
    let width = (WIDTH as u16).min(area.width);
    let x = area.x + (area.width - width) / 2;
    frame.render_widget(Paragraph::new(lines), Rect::new(x, area.y, width, area.height.min(HEIGHT)));
}

/// The tallest keyboard that fits in `rows`, or 0 when none does.
pub fn height_for(rows: u16) -> u16 {
    match rows {
        r if r >= HEIGHT => HEIGHT,
        r if r >= COMPACT_HEIGHT => COMPACT_HEIGHT,
        _ => 0,
    }
}
