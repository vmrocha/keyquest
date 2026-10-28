//! Records, speed history and a per-key accuracy heatmap.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Axis, Block, BorderType, Chart, Dataset, GraphType, Paragraph};

use super::keyboard;
use super::widgets::{self, Hints};
use crate::app::App;
use crate::lessons::MAX_STARS;
use crate::storage::Profile;
use crate::theme::theme;
use crate::typing::KeyStat;

const HISTORY: usize = 30;
/// Keys typed fewer times than this show as "no data".
const MIN_SAMPLES: u32 = 5;

pub fn draw(frame: &mut Frame, area: Rect, app: &App) -> Hints {
    let s = app.strings();
    let Some(profile) = app.current() else { return Vec::new() };
    let keyboard_height = if area.height >= 26 { keyboard::HEIGHT + 3 } else { 0 };
    let [header, _, tiles, _, chart, _, heatmap] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(4),
        Constraint::Length(1),
        Constraint::Min(6),
        Constraint::Length(1),
        Constraint::Length(keyboard_height),
    ])
    .areas(area);

    let title = Line::from(vec![widgets::heading(s.stats_title), widgets::dim(format!("  ·  {}", profile.name))]);
    frame.render_widget(Paragraph::new(title), header);

    let number = |value: Option<f64>| value.map_or("-".to_string(), |v| widgets::round(v).to_string());
    let values = [
        (profile.tests.len().to_string(), s.tests_label, theme().text),
        (number(profile.best_wpm()), s.best_wpm, theme().accent),
        (number(profile.recent_average(10)), s.average, theme().blue),
        (format!("{}/{MAX_STARS}", profile.total_stars()), s.stars_label, theme().yellow),
    ];
    let areas = Layout::horizontal([Constraint::Fill(1); 4]).spacing(1).split(tiles);
    for ((value, label, color), area) in values.into_iter().zip(areas.iter()) {
        let block = Block::bordered().border_type(BorderType::Rounded).border_style(Style::new().fg(theme().border));
        let lines = vec![
            Line::from(Span::styled(value, Style::new().fg(color).bold())),
            Line::from(widgets::dim(label)),
        ];
        frame.render_widget(Paragraph::new(lines).centered().block(block), *area);
    }

    draw_history(frame, chart, profile, s.history, s.no_tests);

    if keyboard_height > 0 {
        let [title, board, legend] =
            Layout::vertical([Constraint::Length(2), Constraint::Length(keyboard::HEIGHT), Constraint::Length(1)])
                .areas(heatmap);
        let heading = Span::styled(s.key_accuracy, Style::new().fg(theme().subtext).bold());
        frame.render_widget(Paragraph::new(heading).centered(), title);
        keyboard::render(frame, board, |key| heat(profile.keys.get(&key)));
        let mut swatches = Vec::new();
        for (color, text) in [(theme().green, "97%+".to_string()), (theme().yellow, "92%+".into()), (theme().orange, "85%+".into()), (theme().red, format!("{} 85%", s.below))] {
            swatches.push(Span::styled("  ", Style::new().bg(color)));
            swatches.push(widgets::dim(format!(" {text}     ")));
        }
        frame.render_widget(Paragraph::new(Line::from(swatches)).centered(), legend);
    }
    vec![("esc", s.h_back)]
}

fn draw_history(frame: &mut Frame, area: Rect, profile: &Profile, title: &str, empty: &str) {
    let recent = &profile.tests[profile.tests.len().saturating_sub(HISTORY)..];
    if recent.is_empty() {
        let block = widgets::panel(title);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let middle = Rect::new(inner.x, inner.y + inner.height / 2, inner.width, 1);
        frame.render_widget(Paragraph::new(widgets::dim(empty)).centered(), middle);
        return;
    }
    let points: Vec<(f64, f64)> = recent.iter().enumerate().map(|(i, test)| (i as f64 + 1.0, test.wpm)).collect();
    let peak = points.iter().map(|p| p.1).fold(10.0, f64::max);
    let top = (peak / 10.0).ceil() * 10.0;
    let count = points.len() as f64;
    let graph = if points.len() > 1 { GraphType::Line } else { GraphType::Scatter };
    let dataset = Dataset::default()
        .marker(Marker::Braille)
        .graph_type(graph)
        .style(Style::new().fg(theme().accent))
        .data(&points);
    let axis_style = Style::new().fg(theme().faint);
    let chart = Chart::new(vec![dataset])
        .block(widgets::panel(title))
        .x_axis(Axis::default().bounds([1.0, count.max(2.0)]).style(axis_style).labels(["1".to_string(), format!("{count:.0}")]))
        .y_axis(Axis::default().bounds([0.0, top]).style(axis_style).labels([
            "0".to_string(),
            format!("{:.0}", top / 2.0),
            format!("{top:.0}"),
        ]));
    frame.render_widget(chart, area);
}

fn heat(stat: Option<&KeyStat>) -> Style {
    let Some(stat) = stat.filter(|stat| stat.total >= MIN_SAMPLES) else {
        return Style::new().fg(theme().inactive).bg(theme().sunken);
    };
    let accuracy = 100.0 * (1.0 - f64::from(stat.misses) / f64::from(stat.total));
    theme().on(heat_color(accuracy)).bold()
}

fn heat_color(accuracy: f64) -> Color {
    match accuracy {
        a if a >= 97.0 => theme().green,
        a if a >= 92.0 => theme().yellow,
        a if a >= 85.0 => theme().orange,
        _ => theme().red,
    }
}
