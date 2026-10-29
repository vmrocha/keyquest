//! keyquest: learn to touch type in the terminal, then practice your speed.

mod app;
mod celebrate;
mod i18n;
mod keyboard;
mod lessons;
mod palette;
mod rng;
mod storage;
mod theme;
mod typing;
mod ui;
mod words;

use std::io;

fn main() -> io::Result<()> {
    let path = storage::data_path();
    let (store, notice) = storage::Store::load(&path);
    // Ask for the colors before anything else reads from the terminal.
    theme::init(palette::query());
    let mut app = app::App::new(store, path, notice);
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
