//! The on-screen keyboard.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::keyboard::ROWS;

const KEY_WIDTH: usize = 5;
/// How far each row is shifted right, like a real keyboard's stagger.
const ROW_OFFSETS: [usize; 3] = [0, 2, 5];
const WIDTH: u16 = (ROW_OFFSETS[2] + 10 * (KEY_WIDTH + 1) - 1) as u16;
pub const HEIGHT: u16 = 7;

/// Draws the keyboard centered in `area`. `style` picks each key's look; the
/// space bar is passed as `' '`.
pub fn render(frame: &mut Frame, area: Rect, style: impl Fn(char) -> Style) {
    let mut lines = Vec::new();
    for (row, keys) in ROWS.iter().enumerate() {
        let mut spans = vec![Span::raw(" ".repeat(ROW_OFFSETS[row]))];
        for (i, key) in keys.chars().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }
            let label = key.to_uppercase().to_string();
            spans.push(Span::styled(format!("{label:^KEY_WIDTH$}"), style(key)));
        }
        lines.push(Line::from(spans));
        lines.push(Line::default());
    }
    // The space bar runs under C to M.
    let start = ROW_OFFSETS[2] + 2 * (KEY_WIDTH + 1);
    let width = 5 * (KEY_WIDTH + 1) - 1;
    lines.push(Line::from(vec![Span::raw(" ".repeat(start)), Span::styled(" ".repeat(width), style(' '))]));

    let width = WIDTH.min(area.width);
    let x = area.x + (area.width - width) / 2;
    frame.render_widget(Paragraph::new(lines), Rect::new(x, area.y, width, area.height.min(HEIGHT)));
}
