//! The results screen after a lesson or speed test.

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, BorderType, Chart, Dataset, GraphType, Paragraph};

use super::widgets::{self, Hints};
use crate::app::{App, Mode, Results};
use crate::keyboard::finger;
use crate::lessons::{LESSONS, star_thresholds};
use crate::theme::theme;

const TILE_WIDTH: u16 = 24;
const CHART_HEIGHT: u16 = 9;

pub fn draw(frame: &mut Frame, area: Rect, app: &App, results: &Results) -> Hints {
    let s = app.strings();
    let kids = app.settings().kids;
    let feedback = s.feedback(kids);
    let lesson = match results.mode {
        Mode::Lesson(index) => Some(index),
        Mode::Speed(_) => None,
    };
    let passed = results.stars > 0;
    let chart_height = if results.samples.len() >= 2 && area.height >= 30 { CHART_HEIGHT } else { 0 };
    let [title, _, tiles, _, stars, message, _, details, missed, _, chart] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(6),
        Constraint::Length(1),
        Constraint::Length(if lesson.is_some() { 1 } else { 0 }),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(chart_height),
    ])
    .flex(Flex::Center)
    .areas(area);

    let (heading, color) = match lesson {
        Some(_) if passed => (feedback.lesson_passed, theme().green),
        Some(_) => (feedback.lesson_failed, theme().orange),
        None => (feedback.times_up, theme().accent),
    };
    let mut line = vec![Span::styled(heading, Style::new().fg(color).bold())];
    if results.personal_best {
        line.push(Span::raw("   "));
        line.push(Span::styled(format!(" {} ", s.personal_best), theme().on(theme().yellow).bold()));
    }
    frame.render_widget(Paragraph::new(Line::from(line)).centered(), title);

    let [wpm_tile, accuracy_tile] = Layout::horizontal([Constraint::Length(TILE_WIDTH); 2])
        .spacing(2)
        .flex(Flex::Center)
        .areas(tiles);
    tile(frame, wpm_tile, widgets::round(results.wpm), s.wpm, theme().accent);
    tile(frame, accuracy_tile, widgets::round(results.accuracy), &format!("% {}", s.accuracy), accuracy_color(results.accuracy));

    if lesson.is_some() {
        frame.render_widget(Paragraph::new(Line::from(widgets::stars(results.stars, 3))).centered(), stars);
    }
    let text = match lesson {
        Some(index) if passed && index + 1 < LESSONS.len() => feedback.next_unlocked.to_string(),
        Some(_) if passed => feedback.all_done.to_string(),
        Some(_) => feedback.unlock_hint.replace("{n}", &star_thresholds(kids)[0].to_string()),
        None => praise(feedback.praise, results.accuracy).to_string(),
    };
    frame.render_widget(Paragraph::new(text).style(Style::new().fg(theme().text)).centered(), message);

    // Label first, so "errors 1" reads fine without plural rules.
    let facts = format!(
        "{} {}  ·  {} {}  ·  {} {}  ·  {:.0}s",
        s.raw,
        widgets::round(results.raw_wpm),
        s.correct,
        results.correct,
        s.errors,
        results.errors,
        results.seconds
    );
    frame.render_widget(Paragraph::new(widgets::dim(facts)).centered(), details);

    if !results.missed.is_empty() {
        let mut line = vec![widgets::dim(format!("{}   ", s.practice_keys))];
        for &key in &results.missed {
            let label = if key == ' ' { s.space.to_string() } else { key.to_uppercase().to_string() };
            line.push(Span::styled(label, Style::new().fg(widgets::finger_color(finger(key))).bold()));
            line.push(Span::raw("  "));
        }
        frame.render_widget(Paragraph::new(Line::from(line)).centered(), missed);
    }

    if chart_height > 0 {
        draw_chart(frame, chart, &results.samples, s.wpm_over_time);
    }

    match lesson {
        Some(index) if passed && index + 1 < LESSONS.len() => {
            vec![("⏎", s.h_next), ("tab", s.h_retry), ("esc", s.h_back)]
        }
        Some(_) if passed => vec![("⏎", s.h_speed_test), ("tab", s.h_retry), ("esc", s.h_back)],
        Some(_) => vec![("⏎", s.h_retry), ("esc", s.h_back)],
        None => vec![("⏎", s.h_again), ("esc", s.h_back)],
    }
}

fn tile(frame: &mut Frame, area: Rect, value: u32, label: &str, color: ratatui::style::Color) {
    let block = Block::bordered().border_type(BorderType::Rounded).border_style(Style::new().fg(theme().border));
    let mut lines = widgets::big_number(value, color);
    lines.push(Line::from(widgets::dim(label.to_string())));
    frame.render_widget(Paragraph::new(lines).centered().block(block), area);
}

fn accuracy_color(accuracy: f64) -> ratatui::style::Color {
    match accuracy {
        a if a >= 95.0 => theme().green,
        a if a >= 85.0 => theme().yellow,
        _ => theme().red,
    }
}

fn praise(options: [&'static str; 3], accuracy: f64) -> &'static str {
    match accuracy {
        a if a >= 97.0 => options[0],
        a if a >= 90.0 => options[1],
        _ => options[2],
    }
}

fn draw_chart(frame: &mut Frame, area: Rect, samples: &[(f64, f64)], title: &str) {
    let last = samples.last().map_or(1.0, |p| p.0);
    let peak = samples.iter().map(|p| p.1).fold(10.0, f64::max);
    let top = (peak / 10.0).ceil() * 10.0;
    let dataset = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::new().fg(theme().accent))
        .data(samples);
    let axis_style = Style::new().fg(theme().faint);
    let chart = Chart::new(vec![dataset])
        .block(widgets::panel(title))
        .x_axis(Axis::default().bounds([1.0, last]).style(axis_style).labels(["1s".to_string(), format!("{last:.0}s")]))
        .y_axis(Axis::default().bounds([0.0, top]).style(axis_style).labels([
            "0".to_string(),
            format!("{:.0}", top / 2.0),
            format!("{top:.0}"),
        ]));
    frame.render_widget(chart, area);
}
