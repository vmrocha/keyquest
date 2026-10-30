# keyquest

Learn to type. Then type faster.

A terminal typing tutor for any age. Step-by-step lessons teach touch typing from the home row to numbers and symbols, and timed speed tests track how fast you get. It speaks English and Brazilian Portuguese, and it picks up your terminal's colors.

![A lesson in progress](docs/screenshots/lesson.png)

## Features

- **23 lessons** on a quest map of six worlds, from the home row up to capitals, punctuation, numbers and symbols, with an on-screen keyboard that shows which finger (and which Shift) to use
- **Word Rain**, a game where you pop falling words before they land, using only the keys you've learned
- **Combos and cheers** for long runs without mistakes, plus stars and confetti when you pass a lesson
- **Kids mode** with kid-friendly words, shorter lessons and easier stars
- **Speed tests** with real words, in 15, 30, 60 or 120 seconds
- **Stats**: personal bests, a history chart and accuracy per key, so you know what to practice
- **Profiles**, so everyone in the house keeps their own progress
- **English and Português (Brasil)**, for menus, lessons and word lists
- **Matches your terminal theme**, or use the built-in Catppuccin palette

## Screenshots

<table>
  <tr>
    <td><img src="docs/screenshots/menu.png" alt="Main menu"></td>
    <td><img src="docs/screenshots/lessons.png" alt="Lesson list"></td>
  </tr>
  <tr>
    <td><img src="docs/screenshots/results.png" alt="Speed test results"></td>
    <td><img src="docs/screenshots/stats.png" alt="Stats"></td>
  </tr>
</table>

## Themes

The default **Terminal** theme asks your terminal for its colors and builds the palette from them, so keyquest looks at home on light and dark themes alike. Terminals that don't answer get the standard ANSI colors. Prefer a fixed look? Switch to **Catppuccin** in Settings.

<table>
  <tr>
    <td><img src="docs/screenshots/lessons.png" alt="Terminal theme on a light terminal"></td>
    <td><img src="docs/screenshots/lessons-catppuccin.png" alt="Catppuccin theme"></td>
  </tr>
  <tr>
    <td align="center">Terminal (on a light theme)</td>
    <td align="center">Catppuccin</td>
  </tr>
</table>

## Install

On macOS or Linux:

```sh
curl -LsSf https://github.com/vmrocha/keyquest/releases/latest/download/keyquest-installer.sh | sh
```

On Windows, in PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/vmrocha/keyquest/releases/latest/download/keyquest-installer.ps1 | iex"
```

Both put `keyquest` in `~/.local/bin` and add it to your `PATH`. You can also download a binary from the [releases page](https://github.com/vmrocha/keyquest/releases). On macOS, a binary downloaded with a browser is blocked the first time; run `xattr -d com.apple.quarantine keyquest` to allow it.

### From source

You need [Rust](https://rustup.rs) 1.88 or newer.

```sh
git clone https://github.com/vmrocha/keyquest
cd keyquest
cargo install --path .
keyquest
```

Or run it without installing: `cargo run --release`.

## How it works

### Lessons

The lessons sit on a quest map, one world per row, joined into a single path. Finish every lesson in a world to clear it.

| World | Lessons | New keys |
| --- | --- | --- |
| Home Row Meadow | 1 to 5 | `F J`, `D K`, `S L`, `A ;`, `G H` |
| Top Row Hills | 6 to 8 | `E I`, `R U`, `T O` |
| Letter Forest | 9 to 15 | `N M`, `C V`, `W Y`, `B P`, `X Q`, `Z ,`, `.` |
| Capital Castle | 16 to 18 | `Shift` for capitals, then `/ ?`, `' " :` |
| Number Caves | 19 to 20 | `1 2 3 4 5`, `6 7 8 9 0` |
| Symbol Volcano | 21 to 23 | `- ! ( )`, `@ # $ %`, `& * + =` |

Each lesson only uses keys you have learned so far. Capitals and symbols use the Shift key on the other hand, and the keyboard lights up both keys. Accuracy earns stars: 85% for one star (and the next lesson), 93% for two, 98% for three. In Kids mode it's 75%, 85% and 95%.

### Word Rain

Words fall from the top of the screen. Type one to pop it before it reaches the ground; each letter is worth points, and the points grow with the level. A wrong key starts the word you're typing over, so mashing keys doesn't help. Every 10 words the rain gets faster. You have 3 hearts, and each word that lands costs one. Word Rain only uses keys from the lessons you've passed, so it's open from the start and grows with you.

### Keys

| Key | Action |
| --- | --- |
| `↑` `↓` (or `j` `k`) | Move |
| `←` `→` (or `h` `l`) | Move along the quest map |
| `⏎` | Select |
| `1` to `7` | Jump to a menu item |
| `space` | Start Word Rain |
| `tab` | Restart the lesson or test |
| `←` `→` | Change the test length before you start typing |
| `ctrl+w` or `alt+backspace` | Delete the last word |
| `n` / `d` | New or delete a profile, on the profile screen |
| `esc` | Back |
| `q` | Quit, from the main menu |

### Settings

- **Kids mode**: kid-friendly words, 80-character lessons instead of 120, easier stars and cheerier messages
- **Language**: English or Português (Brasil)
- **Accents**: Exact needs the accent; Relaxed accepts the plain letter, so `a` works for `á`
- **Speed test length**: 15, 30, 60 or 120 seconds
- **Keyboard in speed tests**: show or hide the on-screen keyboard
- **Theme**: Terminal or Catppuccin

## Your data

Profiles and progress are saved to a JSON file:

- macOS: `~/Library/Application Support/keyquest/profiles.json`
- Linux: `~/.local/share/keyquest/profiles.json`

Set `KEYQUEST_DATA` to use a different file, for example to try things out without touching your progress:

```sh
KEYQUEST_DATA=/tmp/keyquest.json keyquest
```

## License

MIT. See [LICENSE](LICENSE).
