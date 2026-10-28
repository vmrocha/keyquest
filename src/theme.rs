//! Colors. The default theme is built from the terminal's own palette, so the
//! app fits light and dark terminals alike. Catppuccin Mocha is built in.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use ratatui::style::{Color, Style};
use serde::{Deserialize, Serialize};

use crate::palette::{Palette, Rgb};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeChoice {
    #[default]
    Terminal,
    Catppuccin,
}

impl ThemeChoice {
    pub fn name(self) -> &'static str {
        match self {
            Self::Terminal => "Terminal",
            Self::Catppuccin => "Catppuccin",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Terminal => Self::Catppuccin,
            Self::Catppuccin => Self::Terminal,
        }
    }
}

pub struct Theme {
    /// Painted behind everything. `Color::Reset` keeps the terminal's own.
    pub background: Color,
    /// The background as a color, for text drawn on colored blocks. `Color::Reset` when unknown.
    base: Color,
    pub text: Color,
    pub subtext: Color,
    pub muted: Color,
    /// Hints, text still to type, chart axes.
    pub faint: Color,
    /// Empty stars, locked keys, the title's shadow.
    pub inactive: Color,
    pub border: Color,
    /// Selected rows and keycaps.
    pub surface: Color,
    /// Locked keycaps.
    pub sunken: Color,
    pub accent: Color,
    pub highlight: Color,
    pub blue: Color,
    pub green: Color,
    pub yellow: Color,
    pub orange: Color,
    pub red: Color,
    /// One color per finger, in `Finger` order.
    pub fingers: [Color; 9],
    gradient: [Color; 3],
}

impl Theme {
    /// Background-colored text on a block of `color`, as used by badges and the cursor.
    pub fn on(&self, color: Color) -> Style {
        match self.base {
            // Reversing swaps in the terminal's background even when its value is unknown.
            Color::Reset => Style::new().fg(color).bg(Color::Reset).reversed(),
            base => Style::new().fg(base).bg(color),
        }
    }

    /// A color along the title gradient. `t` runs from 0 to 1.
    pub fn gradient(&self, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0) * (self.gradient.len() - 1) as f64;
        let i = (t as usize).min(self.gradient.len() - 2);
        let f = t - i as f64;
        match (self.gradient[i], self.gradient[i + 1]) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => rgb(mix((r1, g1, b1), (r2, g2, b2), f)),
            // Named colors can't be blended, so step between them.
            (a, b) => if f < 0.5 { a } else { b },
        }
    }

    /// Shades between the terminal's background and text, with its hues as accents.
    /// Falls back to the plain ANSI colors for whatever the terminal didn't report.
    fn terminal(palette: Option<Palette>) -> Self {
        let shade = |amount: f64, fallback: Color| {
            palette.map_or(fallback, |p| rgb(mix(p.background, p.foreground, amount)))
        };
        let [red, green, yellow, blue, magenta, cyan, orange, pink, violet] =
            match palette.and_then(|p| p.hues) {
                Some([red, green, yellow, blue, magenta, cyan]) => [
                    red,
                    green,
                    yellow,
                    blue,
                    magenta,
                    cyan,
                    mix(red, yellow, 0.5),
                    mix(red, magenta, 0.5),
                    mix(blue, magenta, 0.5),
                ]
                .map(rgb),
                None => [
                    Color::Red,
                    Color::Green,
                    Color::Yellow,
                    Color::Blue,
                    Color::Magenta,
                    Color::Cyan,
                    Color::LightRed,
                    Color::LightMagenta,
                    Color::LightBlue,
                ],
            };
        Self {
            background: Color::Reset,
            base: palette.map_or(Color::Reset, |p| rgb(p.background)),
            text: Color::Reset,
            subtext: shade(0.78, Color::Reset),
            muted: shade(0.55, Color::DarkGray),
            faint: shade(0.45, Color::DarkGray),
            inactive: shade(0.33, Color::DarkGray),
            border: shade(0.22, Color::DarkGray),
            surface: shade(0.11, Color::Reset),
            sunken: Color::Reset,
            accent: magenta,
            highlight: blue,
            blue,
            green,
            yellow,
            orange,
            red,
            fingers: [magenta, blue, cyan, green, yellow, orange, red, pink, violet],
            gradient: [magenta, blue, cyan],
        }
    }
}

mod mocha {
    use ratatui::style::Color;

    pub const MANTLE: Color = Color::Rgb(0x18, 0x18, 0x25);
    pub const BASE: Color = Color::Rgb(0x1e, 0x1e, 0x2e);
    pub const SURFACE0: Color = Color::Rgb(0x31, 0x32, 0x44);
    pub const SURFACE1: Color = Color::Rgb(0x45, 0x47, 0x5a);
    pub const SURFACE2: Color = Color::Rgb(0x58, 0x5b, 0x70);
    pub const OVERLAY0: Color = Color::Rgb(0x6c, 0x70, 0x86);
    pub const OVERLAY1: Color = Color::Rgb(0x7f, 0x84, 0x9c);
    pub const SUBTEXT: Color = Color::Rgb(0xa6, 0xad, 0xc8);
    pub const TEXT: Color = Color::Rgb(0xcd, 0xd6, 0xf4);
    pub const LAVENDER: Color = Color::Rgb(0xb4, 0xbe, 0xfe);
    pub const BLUE: Color = Color::Rgb(0x89, 0xb4, 0xfa);
    pub const TEAL: Color = Color::Rgb(0x94, 0xe2, 0xd5);
    pub const GREEN: Color = Color::Rgb(0xa6, 0xe3, 0xa1);
    pub const YELLOW: Color = Color::Rgb(0xf9, 0xe2, 0xaf);
    pub const PEACH: Color = Color::Rgb(0xfa, 0xb3, 0x87);
    pub const MAROON: Color = Color::Rgb(0xeb, 0xa0, 0xac);
    pub const RED: Color = Color::Rgb(0xf3, 0x8b, 0xa8);
    pub const MAUVE: Color = Color::Rgb(0xcb, 0xa6, 0xf7);
    pub const PINK: Color = Color::Rgb(0xf5, 0xc2, 0xe7);
}

static CATPPUCCIN: Theme = {
    use mocha::*;
    Theme {
        background: BASE,
        base: BASE,
        text: TEXT,
        subtext: SUBTEXT,
        muted: OVERLAY1,
        faint: OVERLAY0,
        inactive: SURFACE2,
        border: SURFACE1,
        surface: SURFACE0,
        sunken: MANTLE,
        accent: MAUVE,
        highlight: LAVENDER,
        blue: BLUE,
        green: GREEN,
        yellow: YELLOW,
        orange: PEACH,
        red: RED,
        fingers: [MAUVE, BLUE, TEAL, GREEN, YELLOW, PEACH, MAROON, PINK, LAVENDER],
        gradient: [MAUVE, BLUE, TEAL],
    }
};

static TERMINAL: OnceLock<Theme> = OnceLock::new();
static USE_CATPPUCCIN: AtomicBool = AtomicBool::new(false);

/// Builds the terminal theme. Call once, before the first frame.
pub fn init(palette: Option<Palette>) {
    let _ = TERMINAL.set(Theme::terminal(palette));
}

pub fn select(choice: ThemeChoice) {
    USE_CATPPUCCIN.store(choice == ThemeChoice::Catppuccin, Ordering::Relaxed);
}

/// The theme in use.
pub fn theme() -> &'static Theme {
    if USE_CATPPUCCIN.load(Ordering::Relaxed) {
        return &CATPPUCCIN;
    }
    TERMINAL.get_or_init(|| Theme::terminal(None))
}

fn mix(a: Rgb, b: Rgb, amount: f64) -> Rgb {
    let channel = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * amount).round() as u8;
    (channel(a.0, b.0), channel(a.1, b.1), channel(a.2, b.2))
}

fn rgb((r, g, b): Rgb) -> Color {
    Color::Rgb(r, g, b)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIGHT: Palette = Palette {
        foreground: (0x4d, 0x3a, 0x33),
        background: (0xfd, 0xf3, 0xec),
        hues: Some([(0xc4, 0x4f, 0x36), (0, 0x80, 0), (0xc0, 0x90, 0), (0, 0, 0xc0), (0x9c, 0x55, 0x89), (0x3a, 0x88, 0x78)]),
    };

    #[test]
    fn shades_sit_between_background_and_text() {
        let theme = Theme::terminal(Some(LIGHT));
        assert_eq!(theme.background, Color::Reset);
        assert_eq!(theme.surface, Color::Rgb(0xea, 0xdf, 0xd8));
        assert_eq!(theme.accent, Color::Rgb(0x9c, 0x55, 0x89));
        assert_eq!(theme.on(theme.accent), Style::new().fg(Color::Rgb(0xfd, 0xf3, 0xec)).bg(theme.accent));
    }

    #[test]
    fn falls_back_to_ansi_colors() {
        let theme = Theme::terminal(None);
        assert_eq!(theme.accent, Color::Magenta);
        assert_eq!(theme.muted, Color::DarkGray);
        assert_eq!(theme.on(Color::Yellow), Style::new().fg(Color::Yellow).bg(Color::Reset).reversed());
        assert_eq!(theme.gradient(0.0), Color::Magenta);
        assert_eq!(theme.gradient(1.0), Color::Cyan);

        let hues_unknown = Theme::terminal(Some(Palette { hues: None, ..LIGHT }));
        assert_eq!(hues_unknown.accent, Color::Magenta);
        assert_eq!(hues_unknown.base, Color::Rgb(0xfd, 0xf3, 0xec));
    }

    #[test]
    fn gradient_blends_known_colors() {
        assert_eq!(CATPPUCCIN.gradient(0.0), mocha::MAUVE);
        assert_eq!(CATPPUCCIN.gradient(0.5), mocha::BLUE);
        assert_eq!(CATPPUCCIN.gradient(1.0), mocha::TEAL);
    }
}
