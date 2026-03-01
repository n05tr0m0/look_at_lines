use owo_colors::Rgb;
use std::io::IsTerminal;
use std::time::Duration;
use terminal_colorsaurus::{color_scheme, ColorScheme, QueryOptions};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

pub struct Palette {
    pub border: Rgb,
    pub dir: Rgb,
    pub file: Rgb,
    pub exec: Rgb,
}

pub fn detect() -> Theme {
    // Skip the OSC query entirely when there is no controlling terminal —
    // the request would block until the timeout expires (test harnesses,
    // CI runners, pipes).  In those environments Dark is a safe default.
    if !std::io::stderr().is_terminal() {
        return Theme::Dark;
    }
    let mut opts = QueryOptions::default();
    opts.timeout = Duration::from_millis(150);
    match color_scheme(opts) {
        Ok(ColorScheme::Light) => Theme::Light,
        _ => Theme::Dark,
    }
}

pub fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Dark => Palette {
            border: Rgb(0xB2, 0x94, 0xBC),
            dir: Rgb(0x42, 0x7A, 0xB4),
            file: Rgb(0xFF, 0xFF, 0xFF),
            // Gold: visually distinct from plain white files even without bold.
            exec: Rgb(0xFF, 0xD7, 0x00),
        },
        Theme::Light => Palette {
            border: Rgb(0x64, 0x64, 0x64),
            dir: Rgb(0x00, 0x64, 0x00),
            file: Rgb(0x1E, 0x1E, 0x1E),
            exec: Rgb(0xB4, 0x00, 0x00),
        },
    }
}
