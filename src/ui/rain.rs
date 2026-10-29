//! The Word Rain game: the sky, the falling words and the ground.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Clear, Padding, Paragraph};

use super::widgets::{self, Hints};
use crate::app::{App, GameOver, RainPlay};
use crate::rain::{Drop, HEARTS, Rain};
use crate::theme::theme;

const PANEL_WIDTH: u16 = 60;
/// Rows a pop's points rise per second.
const RISE: f64 = 4.0;

pub fn draw(frame: &mut Frame, area: Rect, app: &App, play: &RainPlay) -> Hints {
    let s = app.strings();
    let game = &play.game;
    let best = app.current().map_or(0, |profile| profile.rain_best);
    let [header, _, sky, ground] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1), Constraint::Fill(1), Constraint::Length(1)])
            .areas(area);

    frame.render_widget(Paragraph::new(widgets::heading(s.rain_title)), header);
    let mut status: Vec<Span> = (0..HEARTS)
        .map(|i| match i < game.hearts {
            true => Span::styled("♥ ", Style::new().fg(theme().red)),
            false => Span::styled("♡ ", Style::new().fg(theme().inactive)),
        })
        .collect();
    status.extend([
        widgets::dim(format!("   {} ", s.level)),
        Span::styled(game.level().to_string(), Style::new().fg(theme().text).bold()),
        Span::raw("    "),
        Span::styled(game.score.to_string(), Style::new().fg(theme().accent).bold()),
        widgets::dim(format!(" {}", s.points)),
    ]);
    frame.render_widget(Paragraph::new(Line::from(status)).right_aligned(), header);
    frame.render_widget(Paragraph::new("▀".repeat(ground.width as usize)).style(Style::new().fg(theme().green)), ground);

    for drop in &game.drops {
        draw_drop(frame, sky, drop);
    }
    draw_splashes(frame, sky, game);

    if !game.is_started() {
        let mut lines = vec![
            Line::from(widgets::heading(s.rain_title)),
            Line::default(),
            Line::styled(s.rain_intro[0], Style::new().fg(theme().text)),
            Line::styled(s.rain_intro[1], Style::new().fg(theme().text)),
            Line::default(),
            Line::from(Span::styled(format!(" {} ", s.rain_start), theme().on(theme().accent).bold())),
        ];
        if best > 0 {
            lines.push(Line::default());
            lines.push(Line::from(widgets::dim(format!("{} {best}", s.best))));
        }
        let [middle] = Layout::vertical([Constraint::Length(lines.len() as u16)]).flex(Flex::Center).areas(sky);
        frame.render_widget(Paragraph::new(lines).centered(), middle);
        return vec![("space", s.h_start), ("esc", s.h_back)];
    }

    if let Some(over) = &play.over {
        draw_game_over(frame, sky, app, game, over, best);
        return vec![("⏎", s.h_play_again), ("esc", s.h_back)];
    }
    vec![("esc", s.h_back), ("tab", s.h_restart)]
}

/// Where text of `width` columns sits in the sky for a position from 0 to 1 on each axis.
fn place(sky: Rect, x: f64, y: f64, width: u16) -> Rect {
    let width = width.min(sky.width);
    let column = sky.x + (x * f64::from(sky.width - width)).round() as u16;
    let row = sky.y + (y.clamp(0.0, 1.0) * f64::from(sky.height.saturating_sub(1))).round() as u16;
    Rect::new(column, row, width, 1)
}

fn draw_drop(frame: &mut Frame, sky: Rect, drop: &Drop) {
    // Words turn warmer as they near the ground.
    let color = match drop.y {
        y if y >= 0.8 => theme().red,
        y if y >= 0.55 => theme().orange,
        _ => theme().text,
    };
    let typing = drop.typed > 0;
    let spans: Vec<Span> = drop
        .word
        .iter()
        .enumerate()
        .map(|(i, c)| match i < drop.typed {
            true => Span::styled(c.to_string(), Style::new().fg(theme().green).bold()),
            false if typing => Span::styled(c.to_string(), Style::new().fg(color).bold().underlined()),
            false => Span::styled(c.to_string(), Style::new().fg(color)),
        })
        .collect();
    let at = place(sky, drop.x, drop.y, drop.word.len() as u16);
    frame.render_widget(Paragraph::new(Line::from(spans)), at);
}

/// Points rising from popped words, and a lost heart where a word landed.
fn draw_splashes(frame: &mut Frame, sky: Rect, game: &Rain) {
    for splash in &game.splashes {
        let (text, color, y) = match splash.points {
            0 => ("-♥".to_string(), theme().red, 1.0),
            points => {
                let rise = splash.age * RISE / f64::from(sky.height.max(1));
                (format!("+{points}"), theme().yellow, splash.y - rise)
            }
        };
        let at = place(sky, splash.x, y, text.chars().count() as u16);
        frame.render_widget(Paragraph::new(Span::styled(text, Style::new().fg(color).bold())), at);
    }
}

fn draw_game_over(frame: &mut Frame, sky: Rect, app: &App, game: &Rain, over: &GameOver, best: u32) {
    let s = app.strings();
    let mut heading = vec![Span::styled(s.game_over, Style::new().fg(theme().accent).bold())];
    if over.personal_best {
        heading.push(Span::raw("   "));
        heading.push(Span::styled(format!(" {} ", s.personal_best), theme().on(theme().yellow).bold()));
    }
    let mut lines = vec![Line::from(heading), Line::default()];
    lines.extend(widgets::big_number(game.score, theme().accent));
    lines.push(Line::from(widgets::dim(s.points)));
    lines.push(Line::default());
    let facts = format!(
        "{} {}  ·  {} {}  ·  {:.0}% {}  ·  {} {best}",
        s.words_label,
        game.popped,
        s.level,
        game.level(),
        game.accuracy(),
        s.accuracy,
        s.best
    );
    lines.push(Line::from(widgets::dim(facts)));

    let height = lines.len() as u16 + 4;
    let [panel] = Layout::vertical([Constraint::Length(height)]).flex(Flex::Center).areas(sky);
    let panel = panel.centered_horizontally(Constraint::Length(PANEL_WIDTH.min(sky.width)));
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme().border))
        .padding(Padding::vertical(1))
        .style(Style::new().bg(theme().background));
    frame.render_widget(Clear, panel);
    frame.render_widget(Paragraph::new(lines).centered().block(block), panel);

    if let Some(confetti) = &over.confetti {
        widgets::confetti(frame, frame.area(), confetti, over.age().as_secs_f64());
    }
}
