//! Drawing. Each screen renders into the body area and returns its footer hints.

mod keyboard;
mod lessons;
mod menu;
mod profiles;
mod rain;
mod results;
mod settings;
mod stats;
mod typing;
mod widgets;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Paragraph, Wrap};

use crate::app::{App, Screen};
use crate::theme::{self, theme};

const MIN_WIDTH: u16 = 44;
const MIN_HEIGHT: u16 = 18;

pub fn draw(frame: &mut Frame, app: &App) {
    theme::select(app.settings().theme);
    let area = frame.area();
    frame.render_widget(Block::new().style(Style::new().bg(theme().background).fg(theme().text)), area);
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let message = format!("Make the window at least {MIN_WIDTH}x{MIN_HEIGHT}");
        let middle = Rect::new(area.x, area.y + area.height / 2, area.width, 2);
        frame.render_widget(Paragraph::new(message).centered().wrap(Wrap { trim: true }), middle);
        return;
    }

    let width = area.width.min(widgets::MAX_WIDTH);
    let content = Rect::new(area.x + (area.width - width) / 2, area.y + 1, width, area.height - 2);
    let [body, notice, footer] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(1), Constraint::Length(1)]).areas(content);
    let hints = match &app.screen {
        &Screen::Profiles { selected, confirm_delete } => profiles::draw(frame, body, app, selected, confirm_delete),
        Screen::NewProfile { name, error } => profiles::draw_new(frame, body, app, name, *error),
        &Screen::Menu { selected } => menu::draw(frame, body, app, selected),
        &Screen::Lessons { selected } => lessons::draw(frame, body, app, selected),
        Screen::Typing(typing) => typing::draw(frame, body, app, typing),
        Screen::Results(results) => results::draw(frame, body, app, results),
        Screen::Rain(play) => rain::draw(frame, body, app, play),
        Screen::Stats => stats::draw(frame, body, app),
        &Screen::Settings { selected } => settings::draw(frame, body, app, selected),
    };
    if let Some(text) = &app.notice {
        frame.render_widget(Paragraph::new(text.as_str()).style(Style::new().fg(theme().orange)).centered(), notice);
    }
    frame.render_widget(Paragraph::new(widgets::hints(&hints)).centered(), footer);
}
