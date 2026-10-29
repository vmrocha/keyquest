//! The lesson picker: a quest map with one world per row, joined into a
//! single winding path.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::keyboard;
use super::widgets::{self, Hints};
use crate::app::App;
use crate::keyboard::Key;
use crate::lessons::{LESSONS, MAX_STARS, unlocked_keys};
use crate::quest::{self, WORLDS, runs_right, world_of};
use crate::storage::Profile;
use crate::theme::theme;

const NODE_WIDTH: u16 = 4;
/// Rows per world: the lessons, then their stars.
const WORLD_HEIGHT: u16 = 2;
/// Gaps between lessons to try, widest first, with and without world names.
const LAYOUTS: [(bool, u16); 5] = [(true, 3), (true, 2), (false, 3), (false, 2), (false, 1)];

pub fn draw(frame: &mut Frame, area: Rect, app: &App, selected: usize) -> Hints {
    let s = app.strings();
    let Some(profile) = app.current() else { return Vec::new() };
    let map_height = WORLD_HEIGHT * WORLDS.len() as u16;
    // Drop the blank lines when that makes room for the keyboard, or keeps the map from being cut off.
    let (gap, keyboard_height) = [1, 0]
        .into_iter()
        .map(|gap| (gap, keyboard::height_for(area.height.saturating_sub(map_height + 2 + 3 * gap))))
        .find(|&(_, board)| board > 0)
        .unwrap_or((u16::from(area.height >= map_height + 5), 0));
    let [header, _, map, _, detail, _, board] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(gap),
        Constraint::Length(map_height),
        Constraint::Length(gap),
        Constraint::Length(1),
        Constraint::Length(gap),
        Constraint::Length(keyboard_height),
    ])
    .flex(Flex::Center)
    .areas(area);

    let title = vec![
        widgets::heading(s.lessons_title),
        widgets::dim("  ·  "),
        Span::styled(s.worlds[world_of(selected)], Style::new().fg(theme().text).bold()),
    ];
    frame.render_widget(Paragraph::new(Line::from(title)), header);
    let total = Span::styled(format!("★ {} / {MAX_STARS}", profile.total_stars()), Style::new().fg(theme().yellow));
    frame.render_widget(Paragraph::new(total).right_aligned(), header);

    draw_map(frame, map, app, profile, selected);

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

fn draw_map(frame: &mut Frame, area: Rect, app: &App, profile: &Profile, selected: usize) {
    let s = app.strings();
    let columns = quest::columns();
    let label_width = s.worlds.iter().map(|name| name.chars().count()).max().unwrap_or(0) as u16 + 3;
    // Each lesson takes its node and the gap after it, which leaves room for the last turn.
    let path_width = |link: u16| quest::width() as u16 * (NODE_WIDTH + link);
    let (labels, link) = LAYOUTS
        .into_iter()
        .find(|&(labels, link)| path_width(link) + if labels { label_width } else { 0 } <= area.width)
        .unwrap_or((false, 1));
    let left = area.x + area.width.saturating_sub(path_width(link) + if labels { label_width } else { 0 }) / 2;
    let path_x = if labels { left + label_width } else { left };
    let x = |lesson: usize| path_x + columns[lesson] as u16 * (NODE_WIDTH + link);
    // The first lesson without a star is where the player is on the path.
    let here = (0..LESSONS.len()).find(|&i| profile.lesson_stars(i) == 0);
    let path_style = |to: usize| Style::new().fg(if profile.is_unlocked(to) { theme().accent } else { theme().border });

    for (world, lessons) in WORLDS.iter().enumerate() {
        let y = area.y + WORLD_HEIGHT * world as u16;
        if labels {
            let style = if world == world_of(selected) {
                Style::new().fg(theme().text).bold()
            } else if !profile.is_unlocked(lessons.start) {
                Style::new().fg(theme().inactive)
            } else if quest::is_cleared(profile, world) {
                Style::new().fg(theme().green)
            } else {
                Style::new().fg(theme().muted)
            };
            put(frame, left, y, Span::styled(s.worlds[world], style));
        }

        for i in lessons.clone() {
            let unlocked = profile.is_unlocked(i);
            let style = if i == selected {
                theme().on(if unlocked { theme().accent } else { theme().inactive }).bold()
            } else if !unlocked {
                Style::new().fg(theme().inactive).bg(theme().sunken)
            } else if Some(i) == here {
                Style::new().fg(theme().accent).bg(theme().surface).bold()
            } else {
                Style::new().fg(theme().text).bg(theme().surface).bold()
            };
            put(frame, x(i), y, Span::styled(format!(" {:02} ", i + 1), style));

            let stars = profile.lesson_stars(i);
            if stars > 0 {
                let spans: Vec<Span> = (0..3)
                    .map(|n| match n < stars {
                        true => Span::styled("★", Style::new().fg(theme().yellow)),
                        false => Span::styled("☆", Style::new().fg(theme().inactive)),
                    })
                    .collect();
                put(frame, x(i), y + 1, spans);
            } else if Some(i) == here {
                put(frame, x(i) + 1, y + 1, Span::styled("▲", Style::new().fg(theme().accent).bold()));
            }

            if i + 1 < lessons.end {
                let from = x(i).min(x(i + 1)) + NODE_WIDTH;
                put(frame, from, y, Span::styled("━".repeat(link.into()), path_style(i + 1)));
            }
        }

        // Turn down to the next world at the end of this one.
        let last = lessons.end - 1;
        if last + 1 < LESSONS.len() {
            let run = "━".repeat(usize::from(link - 1));
            let pad = " ".repeat(usize::from(link - 1));
            let (at, turn) = match runs_right(world) {
                true => (x(last) + NODE_WIDTH, [format!("{run}┓"), format!("{pad}┃"), format!("{run}┛")]),
                false => (x(last) - link, [format!("┏{run}"), format!("┃{pad}"), format!("┗{run}")]),
            };
            for (row, text) in turn.into_iter().enumerate() {
                put(frame, at, y + row as u16, Span::styled(text, path_style(last + 1)));
            }
        }
    }
}

/// Draws `line` from column `x` of row `y`, cut off at the edge of the screen.
fn put(frame: &mut Frame, x: u16, y: u16, line: impl Into<Line<'static>>) {
    let line = line.into();
    let area = Rect::new(x, y, line.width() as u16, 1).intersection(frame.area());
    frame.render_widget(Paragraph::new(line), area);
}
