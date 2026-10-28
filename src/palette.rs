//! Asks the terminal for its colors, so the default theme can match it.

use std::io::{self, IsTerminal};

use ratatui::crossterm::terminal;

pub type Rgb = (u8, u8, u8);

/// The terminal's colors, as far as it was willing to tell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub foreground: Rgb,
    pub background: Rgb,
    /// ANSI colors 1 to 6: red, green, yellow, blue, magenta and cyan.
    pub hues: Option<[Rgb; 6]>,
}

/// How long to wait for each reply. Terminals that don't support a query
/// answer the status report sent after it, so this only matters for very
/// slow connections.
const TIMEOUT_MS: u64 = 150;

/// Queries the terminal. Must run before anything else reads input.
pub fn query() -> Option<Palette> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return None;
    }
    terminal::enable_raw_mode().ok()?;
    let palette = query_raw();
    let _ = terminal::disable_raw_mode();
    palette
}

fn query_raw() -> Option<Palette> {
    let background = color("\x1b]11;?\x1b\\")?;
    let foreground = color("\x1b]10;?\x1b\\")?;
    let mut hues = [(0, 0, 0); 6];
    let mut known = true;
    for (i, hue) in hues.iter_mut().enumerate() {
        match color(&format!("\x1b]4;{};?\x1b\\", i + 1)) {
            Some(rgb) => *hue = rgb,
            None => {
                known = false;
                break;
            }
        }
    }
    Some(Palette { foreground, background, hues: known.then_some(hues) })
}

fn color(query: &str) -> Option<Rgb> {
    parse(&xterm_query::query_osc(query, TIMEOUT_MS).ok()?)
}

/// Reads the color out of a reply such as `]11;rgb:fdfd/f3f3/ecec`.
fn parse(reply: &str) -> Option<Rgb> {
    let start = reply.find("rgb")?;
    let spec = reply[start + 3..].trim_start_matches('a').strip_prefix(':')?;
    let mut channels = spec.split('/').map(channel);
    Some((channels.next()??, channels.next()??, channels.next()??))
}

/// Scales a channel of one to four hex digits to 0..=255.
fn channel(text: &str) -> Option<u8> {
    let digits: String = text.chars().take_while(char::is_ascii_hexdigit).collect();
    if digits.is_empty() || digits.len() > 4 {
        return None;
    }
    let value = u32::from_str_radix(&digits, 16).ok()?;
    let max = (1 << (4 * digits.len())) - 1;
    Some(((value * 255 + max / 2) / max) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_replies() {
        assert_eq!(parse("]11;rgb:fdfd/f3f3/ecec\x1b"), Some((0xfd, 0xf3, 0xec)));
        assert_eq!(parse("]4;1;rgb:c4/4f/36\x07"), Some((0xc4, 0x4f, 0x36)));
        assert_eq!(parse("]10;rgba:ffff/0000/8080/ffff\x1b"), Some((255, 0, 0x80)));
        assert_eq!(parse("]11;rgb:f/0/8"), Some((255, 0, 136)));
    }

    #[test]
    fn rejects_garbage() {
        assert_eq!(parse("[0n"), None);
        assert_eq!(parse("]11;rgb:zz/00/00"), None);
        assert_eq!(parse("]11;rgb:ffff/ffff"), None);
    }
}
