//! The main menu.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};

use super::widgets::{self, Hints};
use crate::app::{App, MENU_ITEMS};
use crate::lessons::MAX_STARS;
use crate::theme::theme;

const LIST_WIDTH: u16 = 56;

pub fn draw(frame: &mut Frame, area: Rect, app: &App, selected: usize) -> Hints {
    let s = app.strings();
    let Some(profile) = app.current() else { return Vec::new() };
    let [banner, tagline, _, hello, summary, _, list] = Layout::vertical([
        Constraint::Length(widgets::banner_height(area)),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(2 * MENU_ITEMS as u16),
    ])
    .flex(Flex::Center)
    .areas(area);

    widgets::banner(frame, banner);
    frame.render_widget(Paragraph::new(widgets::dim(s.tagline)).centered(), tagline);
    let greeting = Span::styled(s.hello.replace("{name}", &profile.name), Style::new().fg(theme().text).bold());
    frame.render_widget(Paragraph::new(greeting).centered(), hello);

    let mut facts = vec![Span::styled(format!("★ {}/{MAX_STARS}", profile.total_stars()), Style::new().fg(theme().yellow))];
    if let Some(best) = profile.best_wpm() {
        facts.push(widgets::dim(format!("  ·  {} {} {}", s.best, widgets::round(best), s.wpm)));
    }
    facts.push(widgets::dim(format!("  ·  {}", s.tests_count.replace("{n}", &profile.tests.len().to_string()))));
    frame.render_widget(Paragraph::new(Line::from(facts)).centered(), summary);

    let list = list.centered_horizontally(Constraint::Length(LIST_WIDTH));
    for (i, (label, description)) in s.menu.iter().enumerate() {
        let row = Rect::new(list.x, list.y + 2 * i as u16, list.width, 2);
        let active = i == selected;
        if active {
            frame.render_widget(Block::new().style(Style::new().bg(theme().surface)), row);
        }
        let bar = || if active { Span::styled("▌ ", Style::new().fg(theme().accent)) } else { Span::raw("  ") };
        let label_color = if active { theme().accent } else { theme().text };
        let lines = vec![
            Line::from(vec![
                bar(),
                Span::styled(format!("{} ", i + 1), Style::new().fg(theme().faint)),
                Span::styled(*label, Style::new().fg(label_color).bold()),
            ]),
            Line::from(vec![bar(), Span::raw("  "), widgets::dim(*description)]),
        ];
        frame.render_widget(Paragraph::new(lines), row);
    }
    vec![("↑↓", s.h_move), ("⏎", s.h_select), ("esc", s.h_back), ("q", s.h_quit)]
}
