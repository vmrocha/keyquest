//! The typing screen, shared by lessons and speed tests.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph};

use super::keyboard;
use super::widgets::{self, Hints};
use crate::app::{App, Mode, Typing};
use crate::keyboard::{base_key, finger};
use crate::lessons::{LESSONS, unlocked_keys};
use crate::theme::theme;
use crate::typing::{CharState, Session, wrap};

/// Rows used by the header, progress bar, hint and the gaps between them.
const CHROME: u16 = 5;

/// Height of the text box for `lines` lines: a blank line between each, plus padding and border.
const fn text_height(lines: usize) -> u16 {
    (lines * 2 - 1) as u16 + 4
}

pub fn draw(frame: &mut Frame, area: Rect, app: &App, typing: &Typing) -> Hints {
    let s = app.strings();
    let session = &typing.session;
    let summary = session.summary(Instant::now());
    let wants_keyboard = match typing.mode {
        Mode::Lesson(_) => true,
        Mode::Speed(_) => app.settings().keyboard_in_test,
    };
    // Shrink the text box to two lines before giving up on the keyboard.
    let keyboard_rows = keyboard::HEIGHT + 1;
    let (visible, show_keyboard) = match area.height {
        _ if !wants_keyboard => (3, false),
        h if h >= CHROME + text_height(3) + keyboard_rows => (3, true),
        h if h >= CHROME + text_height(2) + keyboard_rows => (2, true),
        _ => (3, false),
    };
    let (keyboard_height, finger_height) = if show_keyboard { (keyboard::HEIGHT, 1) } else { (0, 0) };
    let [header, gauge, _, text, hint, _, board, finger_line] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(text_height(visible)),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(keyboard_height),
        Constraint::Length(finger_height),
    ])
    .flex(Flex::Center)
    .areas(area);

    let title = match typing.mode {
        Mode::Lesson(index) => {
            let mut spans = vec![widgets::heading(&s.lesson_n.replace("{n}", &(index + 1).to_string())), Span::raw("   ")];
            spans.extend(widgets::keycaps(LESSONS[index], false));
            spans
        }
        Mode::Speed(seconds) if !session.is_started() => vec![
            widgets::heading(s.speed_test),
            Span::styled("   ‹ ", Style::new().fg(theme().accent)),
            Span::styled(format!("{seconds}s"), Style::new().fg(theme().text).bold()),
            Span::styled(" ›", Style::new().fg(theme().accent)),
        ],
        Mode::Speed(seconds) => vec![widgets::heading(s.speed_test), widgets::dim(format!("   {seconds}s"))],
    };
    frame.render_widget(Paragraph::new(Line::from(title)), header);

    let elapsed = summary.seconds;
    let clock = match session.limit() {
        Some(limit) => (limit.as_secs_f64() - elapsed).max(0.0).ceil(),
        None => elapsed.floor(),
    };
    let stats = Line::from(vec![
        Span::styled(format!("{clock:.0}s"), Style::new().fg(theme().text).bold()),
        Span::raw("    "),
        Span::styled(widgets::round(summary.wpm).to_string(), Style::new().fg(theme().accent).bold()),
        widgets::dim(format!(" {}    ", s.wpm)),
        Span::styled(format!("{:.0}%", summary.accuracy), Style::new().fg(theme().green).bold()),
        widgets::dim(format!(" {}", s.accuracy)),
    ]);
    frame.render_widget(Paragraph::new(stats).right_aligned(), header);

    let ratio = match session.limit() {
        Some(limit) => elapsed / limit.as_secs_f64(),
        None => session.progress(),
    };
    let filled = (ratio.clamp(0.0, 1.0) * f64::from(gauge.width)).round() as usize;
    let bar = Line::from(vec![
        Span::styled("━".repeat(filled), Style::new().fg(theme().accent)),
        Span::styled("━".repeat(gauge.width as usize - filled), Style::new().fg(theme().border)),
    ]);
    frame.render_widget(Paragraph::new(bar), gauge);

    draw_text(frame, text, session, visible);

    if !session.is_started() {
        frame.render_widget(Paragraph::new(widgets::dim(s.start_typing)).centered(), hint);
    }

    let next = session.expected().map(base_key);
    if show_keyboard {
        let known = match typing.mode {
            Mode::Lesson(index) => Some(unlocked_keys(index)),
            Mode::Speed(_) => None,
        };
        keyboard::render(frame, board, |key| {
            let color = widgets::finger_color(finger(key));
            if Some(key) == next {
                theme().on(color).bold()
            } else if key == ' ' || known.as_ref().is_none_or(|keys| keys.contains(&key)) {
                Style::new().fg(color).bg(theme().surface)
            } else {
                Style::new().fg(theme().inactive).bg(theme().sunken)
            }
        });
        if let Some(key) = next {
            let label = if key == ' ' { s.space.to_string() } else { key.to_uppercase().to_string() };
            let finger = finger(key);
            let line = Line::from(vec![
                widgets::dim(format!("{}  ", s.next_key)),
                Span::styled(label, Style::new().fg(widgets::finger_color(finger)).bold()),
                widgets::dim(format!("  ·  {}", s.fingers[finger as usize])),
            ]);
            frame.render_widget(Paragraph::new(line).centered(), finger_line);
        }
    }

    let mut hints = vec![("esc", s.h_back), ("tab", s.h_restart)];
    if matches!(typing.mode, Mode::Speed(_)) && !session.is_started() {
        hints.push(("←→", s.h_length));
    }
    hints
}

fn draw_text(frame: &mut Frame, area: Rect, session: &Session, visible: usize) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(theme().border))
        .padding(Padding::new(3, 3, 1, 1));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = wrap(session.target(), inner.width as usize);
    let cursor = session.position();
    let cursor_line = lines.iter().position(|line| line.contains(&cursor)).unwrap_or(lines.len().saturating_sub(1));
    // Keep the cursor on the second line so the next words are always in view.
    let first = cursor_line.saturating_sub(1).min(lines.len().saturating_sub(visible));

    let mut text = Vec::new();
    for (n, range) in lines.iter().skip(first).take(visible).enumerate() {
        if n > 0 {
            text.push(Line::default());
        }
        text.push(Line::from(range.clone().map(|i| char_span(session, i)).collect::<Vec<_>>()));
    }
    frame.render_widget(Paragraph::new(text), inner);
}

fn char_span(session: &Session, index: usize) -> Span<'static> {
    let c = session.target()[index];
    match session.state(index) {
        CharState::Correct => Span::styled(c.to_string(), Style::new().fg(theme().text)),
        CharState::Wrong => {
            let shown = if c == ' ' { '·' } else { c };
            Span::styled(shown.to_string(), Style::new().fg(theme().red).underlined())
        }
        CharState::Cursor => Span::styled(c.to_string(), theme().on(theme().highlight)),
        CharState::Pending => Span::styled(c.to_string(), Style::new().fg(theme().faint)),
    }
}
