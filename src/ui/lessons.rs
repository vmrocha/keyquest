//! The lesson picker.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::keyboard;
use super::widgets::{self, Hints};
use crate::app::{App, LESSON_COLUMNS, LESSON_ROWS};
use crate::keyboard::Key;
use crate::lessons::{LESSONS, MAX_STARS, unlocked_keys};
use crate::theme::theme;

pub fn draw(frame: &mut Frame, area: Rect, app: &App, selected: usize) -> Hints {
    let s = app.strings();
    let Some(profile) = app.current() else { return Vec::new() };
    let keyboard_height = keyboard::height_for(area.height.saturating_sub(LESSON_ROWS as u16 + 5));
    let [header, _, grid, _, detail, _, board] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(LESSON_ROWS as u16),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(keyboard_height),
    ])
    .flex(Flex::Center)
    .areas(area);

    frame.render_widget(Paragraph::new(widgets::heading(s.lessons_title)), header);
    let total = Span::styled(format!("★ {} / {MAX_STARS}", profile.total_stars()), Style::new().fg(theme().yellow));
    frame.render_widget(Paragraph::new(total).right_aligned(), header);

    let columns = Layout::horizontal([Constraint::Fill(1); LESSON_COLUMNS]).spacing(2).split(grid);
    for (i, keys) in LESSONS.iter().enumerate() {
        let column = columns[i / LESSON_ROWS];
        let row = Rect::new(column.x, column.y + (i % LESSON_ROWS) as u16, column.width, 1);
        let unlocked = profile.is_unlocked(i);
        let number_color = if unlocked { theme().muted } else { theme().inactive };
        let mut label = vec![Span::styled(format!("{:02}   ", i + 1), Style::new().fg(number_color))];
        label.extend(widgets::keycaps(keys, !unlocked));
        // Locked rows stay dimmed with no stars; the line below says why.
        let status = if unlocked { widgets::stars(profile.lesson_stars(i), 3) } else { Vec::new() };
        widgets::list_row(frame, row, i == selected, label, status);
    }

    let mut line = vec![
        Span::styled(s.lesson_n.replace("{n}", &(selected + 1).to_string()), Style::new().fg(theme().text).bold()),
        widgets::dim(format!("  ·  {}  ", s.new_keys)),
    ];
    line.extend(widgets::keycaps(LESSONS[selected], false));
    let record = match profile.lessons.get(&selected) {
        _ if !profile.is_unlocked(selected) => s.locked.to_string(),
        Some(r) if r.best_wpm > 0.0 => {
            format!("{} {:.0}%  ·  {} {}", s.best, r.best_accuracy, widgets::round(r.best_wpm), s.wpm)
        }
        Some(r) => format!("{} {:.0}%", s.best, r.best_accuracy),
        None => s.not_tried.to_string(),
    };
    line.push(widgets::dim(format!("  ·  {record}")));
    frame.render_widget(Paragraph::new(Line::from(line)).centered(), detail);

    if keyboard_height > 0 {
        let new_keys = LESSONS[selected];
        let known = unlocked_keys(selected);
        keyboard::render(frame, board, |key| {
            let color = widgets::finger_color(key.finger());
            if key.used_by(new_keys) {
                theme().on(color).bold()
            } else if key == Key::Char(' ') || key.used_by(&known) {
                Style::new().fg(color).bg(theme().surface)
            } else {
                Style::new().fg(theme().inactive).bg(theme().sunken)
            }
        });
    }
    vec![("↑↓←→", s.h_move), ("⏎", s.h_select), ("esc", s.h_back)]
}
