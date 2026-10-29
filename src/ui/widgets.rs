//! Small building blocks shared by the screens.

use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph};

use crate::celebrate::Confetti;
use crate::keyboard::{Finger, SHIFT, finger};
use crate::theme::theme;

pub const MAX_WIDTH: u16 = 80;

/// (key, label) pairs shown in the footer.
pub type Hints = Vec<(&'static str, &'static str)>;

pub fn hints(items: &[(&str, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (key, label)) in items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("   "));
        }
        spans.push(Span::styled(key.to_string(), Style::new().fg(theme().highlight).bold()));
        spans.push(Span::styled(format!(" {label}"), Style::new().fg(theme().faint)));
    }
    Line::from(spans)
}

pub fn heading(text: &str) -> Span<'static> {
    Span::styled(text.to_string(), Style::new().fg(theme().accent).bold())
}

pub fn dim(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::new().fg(theme().muted))
}

pub fn panel(title: &str) -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme().border))
        .title(Span::styled(format!(" {title} "), Style::new().fg(theme().subtext)))
}

pub fn finger_color(f: Finger) -> ratatui::style::Color {
    theme().fingers[f as usize]
}

/// Keys as uppercase letters in their finger's color, e.g. "F J".
pub fn keycaps(keys: &[char], dimmed: bool) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (i, &key) in keys.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw(" "));
        }
        let (label, finger) = match key {
            SHIFT => ("Shift".to_string(), Finger::LeftPinky),
            _ => (key.to_uppercase().to_string(), finger(key)),
        };
        let color = if dimmed { theme().faint } else { finger_color(finger) };
        spans.push(Span::styled(label, Style::new().fg(color).bold()));
    }
    spans
}

/// Earned stars filled, missing ones as outlines, so they differ by shape and not just color.
pub fn stars(count: u8, max: u8) -> Vec<Span<'static>> {
    (0..max)
        .map(|i| {
            let (star, color) = if i < count { ('★', theme().yellow) } else { ('☆', theme().inactive) };
            let gap = if i + 1 < max { " " } else { "" };
            Span::styled(format!("{star}{gap}"), Style::new().fg(color))
        })
        .collect()
}

/// Confetti falling over `area` `seconds` after the burst. Pieces only land
/// on blank cells, so the text underneath stays readable.
pub fn confetti(frame: &mut Frame, area: Rect, confetti: &Confetti, seconds: f64) {
    let buffer = frame.buffer_mut();
    for (x, y, glyph, color) in confetti.pieces(seconds) {
        let column = area.x + (x * f64::from(area.width.saturating_sub(1))).round() as u16;
        let row = area.y + (y * f64::from(area.height)) as u16;
        if let Some(cell) = buffer.cell_mut((column, row))
            && cell.symbol() == " "
        {
            cell.set_char(glyph).set_fg(theme().fingers[color]);
        }
    }
}

/// The color of a streak: plain at first, then warmer at each milestone.
pub fn streak_color(streak: u32) -> ratatui::style::Color {
    match streak {
        s if s >= 50 => theme().accent,
        s if s >= 25 => theme().orange,
        s if s >= 10 => theme().yellow,
        _ => theme().text,
    }
}

/// A full-width list row with an accent bar when selected and optional right-aligned text.
pub fn list_row(frame: &mut Frame, area: Rect, selected: bool, left: Vec<Span<'static>>, right: Vec<Span<'static>>) {
    if selected {
        frame.render_widget(Block::new().style(Style::new().bg(theme().surface)), area);
    }
    let bar = if selected { Span::styled("▌ ", Style::new().fg(theme().accent)) } else { Span::raw("  ") };
    frame.render_widget(Paragraph::new(Line::from([vec![bar], left].concat())), area);
    if !right.is_empty() {
        let right = Line::from([right, vec![Span::raw(" ")]].concat());
        frame.render_widget(Paragraph::new(right).alignment(Alignment::Right), area);
    }
}

const BANNER_WORD: &str = "KEYQUEST";
pub const BANNER_HEIGHT: u16 = 6;

fn glyph(letter: char) -> [&'static str; 6] {
    match letter {
        'K' => ["██╗  ██╗", "██║ ██╔╝", "█████╔╝ ", "██╔═██╗ ", "██║  ██╗", "╚═╝  ╚═╝"],
        'E' => ["███████╗", "██╔════╝", "█████╗  ", "██╔══╝  ", "███████╗", "╚══════╝"],
        'Y' => ["██╗   ██╗", "╚██╗ ██╔╝", " ╚████╔╝ ", "  ╚██╔╝  ", "   ██║   ", "   ╚═╝   "],
        'Q' => [" ██████╗ ", "██╔═══██╗", "██║   ██║", "██║▄▄ ██║", "╚██████╔╝", " ╚══▀▀═╝ "],
        'U' => ["██╗   ██╗", "██║   ██║", "██║   ██║", "██║   ██║", "╚██████╔╝", " ╚═════╝ "],
        'S' => ["███████╗", "██╔════╝", "███████╗", "╚════██║", "███████║", "╚══════╝"],
        'T' => ["████████╗", "╚══██╔══╝", "   ██║   ", "   ██║   ", "   ██║   ", "   ╚═╝   "],
        _ => [""; 6],
    }
}

/// The big gradient title, or a one-line version when space is short.
pub fn banner(frame: &mut Frame, area: Rect) {
    let rows: Vec<String> = (0..6).map(|row| BANNER_WORD.chars().map(|c| glyph(c)[row]).collect()).collect();
    let width = rows[0].chars().count();
    if area.width as usize <= width || area.height < BANNER_HEIGHT {
        let spaced: Vec<Span> = BANNER_WORD
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let color = theme().gradient(i as f64 / (BANNER_WORD.len() - 1) as f64);
                Span::styled(format!("{} ", c.to_ascii_lowercase()), Style::new().fg(color).bold())
            })
            .collect();
        frame.render_widget(Paragraph::new(Line::from(spaced)).centered(), area);
        return;
    }
    let lines: Vec<Line> = rows
        .iter()
        .map(|row| {
            let spans: Vec<Span> = row
                .chars()
                .enumerate()
                .map(|(col, c)| {
                    // The shadow characters sit a little darker than the blocks.
                    let color = if c == '█' || c == '▄' || c == '▀' { theme().gradient(col as f64 / width as f64) } else { theme().inactive };
                    Span::styled(c.to_string(), Style::new().fg(color))
                })
                .collect();
            Line::from(spans)
        })
        .collect();
    frame.render_widget(Paragraph::new(lines).centered(), area);
}

/// Rows needed by [`banner`] for a screen of this height.
pub fn banner_height(area: Rect) -> u16 {
    if area.height >= 28 && area.width > 70 { BANNER_HEIGHT } else { 1 }
}

const DIGITS: [[&str; 3]; 10] = [
    ["█▀█", "█ █", "▀▀▀"],
    ["▀█ ", " █ ", "▀▀▀"],
    ["▀▀█", "█▀▀", "▀▀▀"],
    ["▀▀█", " ▀█", "▀▀▀"],
    ["█ █", "▀▀█", "  ▀"],
    ["█▀▀", "▀▀█", "▀▀▀"],
    ["█▀▀", "█▀█", "▀▀▀"],
    ["▀▀█", "  █", "  ▀"],
    ["█▀█", "█▀█", "▀▀▀"],
    ["█▀█", "▀▀█", "▀▀▀"],
];

/// A number drawn three rows tall.
pub fn big_number(value: u32, color: ratatui::style::Color) -> Vec<Line<'static>> {
    let digits: Vec<usize> = value.to_string().chars().filter_map(|c| c.to_digit(10)).map(|d| d as usize).collect();
    (0..3)
        .map(|row| {
            let text: Vec<&str> = digits.iter().map(|&d| DIGITS[d][row]).collect();
            Line::styled(text.join(" "), Style::new().fg(color))
        })
        .collect()
}

pub fn round(value: f64) -> u32 {
    value.round().max(0.0) as u32
}
