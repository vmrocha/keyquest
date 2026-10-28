//! Picking and creating profiles.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph};

use super::widgets::{self, Hints};
use crate::app::App;
use crate::theme::theme;

const LIST_WIDTH: u16 = 56;
const INPUT_WIDTH: u16 = 40;

pub fn draw(frame: &mut Frame, area: Rect, app: &App, selected: usize, confirm_delete: bool) -> Hints {
    let s = app.strings();
    let profiles = &app.store.profiles;
    let [banner, tagline, _, heading, _, list, _, prompt] = Layout::vertical([
        Constraint::Length(widgets::banner_height(area)),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(profiles.len() as u16 + 1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .flex(Flex::Center)
    .areas(area);

    widgets::banner(frame, banner);
    frame.render_widget(Paragraph::new(widgets::dim(s.tagline)).centered(), tagline);
    frame.render_widget(Paragraph::new(widgets::heading(s.who)).centered(), heading);

    let list = list.centered_horizontally(Constraint::Length(LIST_WIDTH));
    let rows = Layout::vertical(vec![Constraint::Length(1); profiles.len() + 1]).split(list);
    for (i, profile) in profiles.iter().enumerate() {
        let name = Span::styled(profile.name.clone(), Style::new().fg(theme().text).bold());
        let mut right = vec![Span::styled(format!("★ {}", profile.total_stars()), Style::new().fg(theme().yellow))];
        if let Some(best) = profile.best_wpm() {
            right.push(widgets::dim(format!("   {} {}", widgets::round(best), s.wpm)));
        }
        widgets::list_row(frame, rows[i], i == selected, vec![name], right);
    }
    let new = Span::styled(format!("+ {}", s.new_profile), Style::new().fg(theme().green));
    widgets::list_row(frame, rows[profiles.len()], selected == profiles.len(), vec![new], vec![]);

    if confirm_delete && let Some(profile) = profiles.get(selected) {
        let text = s.delete_confirm.replace("{name}", &profile.name);
        frame.render_widget(Paragraph::new(text).style(Style::new().fg(theme().red)).centered(), prompt);
    }
    vec![
        ("↑↓", s.h_move),
        ("⏎", s.h_select),
        ("n", s.h_new),
        ("d", s.h_delete),
        ("l", s.h_other_language),
        ("q", s.h_quit),
    ]
}

pub fn draw_new(frame: &mut Frame, area: Rect, app: &App, name: &str, error: Option<&str>) -> Hints {
    let s = app.strings();
    let [banner, tagline, _, heading, _, prompt, input, help] = Layout::vertical([
        Constraint::Length(widgets::banner_height(area)),
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(1),
    ])
    .flex(Flex::Center)
    .areas(area);

    widgets::banner(frame, banner);
    frame.render_widget(Paragraph::new(widgets::dim(s.tagline)).centered(), tagline);
    frame.render_widget(Paragraph::new(widgets::heading(s.new_profile)).centered(), heading);
    frame.render_widget(Paragraph::new(s.name_prompt).style(Style::new().fg(theme().text)).centered(), prompt);

    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme().accent))
        .padding(Padding::horizontal(1));
    let text = Line::from(vec![
        Span::styled(name.to_string(), Style::new().fg(theme().text).bold()),
        Span::styled(" ", Style::new().bg(theme().highlight)),
    ]);
    let input = input.centered_horizontally(Constraint::Length(INPUT_WIDTH));
    frame.render_widget(Paragraph::new(text).block(block), input);

    let help_line = match error {
        Some(error) => Span::styled(error.to_string(), Style::new().fg(theme().red)),
        None => widgets::dim(s.name_help),
    };
    frame.render_widget(Paragraph::new(help_line).centered(), help);
    vec![("⏎", s.h_create), ("esc", s.h_back)]
}
