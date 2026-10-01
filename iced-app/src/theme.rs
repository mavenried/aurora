use iced::{Color, Font};

/// The runtime-swappable 10-color palette, mirroring `aurora_protocol::Theme`.
/// Defaults match the Catppuccin Mocha values hardcoded in `app/ui/Types.slint`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub bgd0: Color,
    pub bgd1: Color,
    pub bgd2: Color,
    pub bgd3: Color,
    pub bgd4: Color,
    pub txt1: Color,
    pub txt2: Color,
    pub acct: Color,
    pub srch: Color,
    pub btns: Color,
    pub follow_art_colorway: bool,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            bgd0: hex("#11111b"),
            bgd1: hex("#181825"),
            bgd2: hex("#1e1e2e"),
            bgd3: hex("#313244"),
            bgd4: hex("#45475a"),
            txt1: hex("#cdd6f4"),
            txt2: hex("#a6adc8"),
            acct: hex("#cba6f7"),
            srch: hex("#45475a"),
            btns: hex("#cdd6f4"),
            follow_art_colorway: false,
        }
    }
}

impl From<aurora_protocol::Theme> for Palette {
    fn from(t: aurora_protocol::Theme) -> Self {
        Self {
            bgd0: hex(&t.bgd0),
            bgd1: hex(&t.bgd1),
            bgd2: hex(&t.bgd2),
            bgd3: hex(&t.bgd3),
            bgd4: hex(&t.bgd4),
            txt1: hex(&t.txt1),
            txt2: hex(&t.txt2),
            acct: hex(&t.acct),
            srch: hex(&t.srch),
            btns: hex(&t.btns),
            follow_art_colorway: t.follow_art_colorway,
        }
    }
}

fn hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(0);
    Color::from_rgb8(r, g, b)
}

pub const BODY_FONT: Font = Font::with_name("Noto Sans JP");
