//! Per-profile settings.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::{Paragraph, Wrap};

use super::widgets::{self, Hints};
use crate::app::{App, SETTINGS_ITEMS};
use crate::theme::theme;

const LIST_WIDTH: u16 = 60;

pub fn draw(frame: &mut Frame, area: Rect, app: &App, selected: usize) -> Hints {
    let s = app.strings();
    let settings = app.settings();
    let [header, _, list, _, help] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2 * SETTINGS_ITEMS as u16),
        Constraint::Length(1),
        Constraint::Length(3),
    ])
    .flex(Flex::Center)
    .areas(area);

    frame.render_widget(Paragraph::new(widgets::heading(s.settings_title)).centered(), header);

    let values = [
        settings.language.name().to_string(),
        if settings.relaxed_accents { s.accents_relaxed } else { s.accents_exact }.to_string(),
        format!("{}s", settings.test_seconds),
        if settings.keyboard_in_test { s.on } else { s.off }.to_string(),
        settings.theme.name().to_string(),
    ];
    let list = list.centered_horizontally(Constraint::Length(LIST_WIDTH));
    for (i, (name, value)) in s.setting_names.iter().zip(values).enumerate() {
        let row = Rect::new(list.x, list.y + 2 * i as u16, list.width, 1);
        let active = i == selected;
        let value = if active {
            vec![
                Span::styled("‹ ", Style::new().fg(theme().accent)),
                Span::styled(value, Style::new().fg(theme().text).bold()),
                Span::styled(" ›", Style::new().fg(theme().accent)),
            ]
        } else {
            vec![Span::styled(value, Style::new().fg(theme().subtext))]
        };
        widgets::list_row(frame, row, active, vec![Span::styled(*name, Style::new().fg(theme().text))], value);
    }

    let help = help.centered_horizontally(Constraint::Length(LIST_WIDTH));
    let text = Paragraph::new(widgets::dim(s.setting_help[selected])).centered().wrap(Wrap { trim: true });
    frame.render_widget(text, help);
    vec![("↑↓", s.h_move), ("←→", s.h_change), ("esc", s.h_back)]
}
