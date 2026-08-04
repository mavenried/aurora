/// The runtime-swappable 10-color palette, mirroring `aurora_protocol::Theme`.
/// Defaults match the Catppuccin Mocha values hardcoded in `app/ui/Types.slint`.
#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub bgd0: String,
    pub bgd1: String,
    pub bgd2: String,
    pub bgd3: String,
    pub bgd4: String,
    pub txt1: String,
    pub txt2: String,
    pub acct: String,
    pub srch: String,
    pub btns: String,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            bgd0: "#11111b".into(),
            bgd1: "#181825".into(),
            bgd2: "#1e1e2e".into(),
            bgd3: "#313244".into(),
            bgd4: "#45475a".into(),
            txt1: "#cdd6f4".into(),
            txt2: "#a6adc8".into(),
            acct: "#cba6f7".into(),
            srch: "#45475a".into(),
            btns: "#cdd6f4".into(),
        }
    }
}

impl From<aurora_protocol::Theme> for Palette {
    fn from(t: aurora_protocol::Theme) -> Self {
        Self {
            bgd0: t.bgd0,
            bgd1: t.bgd1,
            bgd2: t.bgd2,
            bgd3: t.bgd3,
            bgd4: t.bgd4,
            txt1: t.txt1,
            txt2: t.txt2,
            acct: t.acct,
            srch: t.srch,
            btns: t.btns,
        }
    }
}

/// Builds a stylesheet for the whole app from the current palette. Reloaded
/// into the shared CssProvider whenever a `Theme` response arrives.
pub fn css(p: &Palette) -> String {
    format!(
        r#"
window {{ background-color: {bgd0}; }}

.panel-bg {{ background-color: {bgd1}; }}
.card-bg {{ background-color: {bgd2}; }}
.raised-bg {{ background-color: {bgd3}; }}

.txt1 {{ color: {txt1}; }}
.txt2 {{ color: {txt2}; }}
.accent {{ color: {acct}; }}

.title-15 {{ font-size: 15px; }}
.title-16 {{ font-size: 16px; }}
.title-18 {{ font-size: 18px; }}
.title-20 {{ font-size: 20px; }}
.subtle-14 {{ font-size: 14px; }}
.subtle-13 {{ font-size: 13px; }}
.subtle-12 {{ font-size: 12px; }}

/* Consistent radius scale: 8px for interactive rows/controls, 16px for
   larger card/dialog surfaces, 999px (fully round) for pills and circles. */

.song-row {{ background-color: transparent; border-radius: 8px; }}
.song-row:hover {{ background-color: {bgd3}; }}
.song-row.selected {{ background-color: {bgd4}; }}
.song-row.selected:hover {{ background-color: {bgd4}; }}

.accent-bar {{ background-color: transparent; min-width: 4px; border-radius: 999px; }}
.accent-bar.selected {{ background-color: {acct}; }}
.accent-bar.playing {{ background-color: alpha({acct}, 0.6); }}

.queue-row {{ background-color: {bgd2}; border-radius: 8px; }}
.queue-row:hover {{ background-color: {bgd3}; }}

.sidebar-row {{ padding: 8px 10px; border-radius: 8px; }}
.sidebar-row:hover {{ background-color: {bgd2}; }}
.sidebar-row.active {{ background-color: {bgd2}; }}
.sidebar-row.active label {{ color: {acct}; }}
.sidebar-row image {{ color: {txt2}; }}
.sidebar-row.active image {{ color: {acct}; }}

.card {{ background-color: {bgd2}; border-radius: 8px; }}
.card:hover {{ background-color: {bgd3}; }}

.thumb {{ border-radius: 6px; }}

separator {{ background-color: {bgd3}; min-height: 1px; }}

.pill-btn {{
  background-color: {bgd3};
  color: {txt1};
  border-radius: 999px;
  padding: 8px 18px;
  min-height: 0;
}}
.pill-btn:hover {{ background-color: {bgd4}; }}
.pill-btn.accent {{ background-color: {acct}; color: {bgd0}; }}
.pill-btn.accent:hover {{ background-color: alpha({acct}, 0.85); }}
.pill-btn.active {{ background-color: {bgd1}; color: {acct}; }}

/* Transport controls: prev/next/shuffle/repeat/like stay flat until
   hovered or toggled active; play/pause always carries a filled circle
   so it reads as the primary action (Amberol-style hierarchy). */
.circle-btn {{
  border-radius: 999px;
  background-color: transparent;
  color: {btns};
  padding: 0;
  border: none;
  box-shadow: none;
}}
.circle-btn:hover {{ background-color: {bgd3}; }}
.circle-btn:active {{ background-color: {bgd4}; }}
.circle-btn.active {{ color: {acct}; background-color: alpha({acct}, 0.16); }}

.play-btn {{ background-color: {bgd3}; color: {txt1}; }}
.play-btn:hover {{ background-color: {bgd4}; }}
.play-btn:active {{ background-color: {bgd4}; }}

.search-bar {{
  background-color: {bgd1};
  border-radius: 999px;
  padding: 8px 16px;
  min-height: 0;
}}

entry {{
  background-color: transparent;
  color: {txt1};
  border: none;
  box-shadow: none;
  min-height: 0;
}}
entry image {{ color: {srch}; }}

entry.dialog-entry {{
  background-color: {bgd3};
  border-radius: 8px;
  padding: 8px 12px;
}}

/* GtkScale CSS nodes are `scale > trough > {{slider, highlight}}`. The
   slider is drawn larger than the trough and centered on it via a
   matching negative margin (trough 4px, slider 16px -> -6px each side);
   all default Adwaita borders/shadows are stripped so only our flat
   trough+highlight+knob render. */
scale {{ min-height: 20px; }}
scale trough {{
  background-color: {bgd3};
  min-height: 4px;
  border-radius: 999px;
  border: none;
  box-shadow: none;
  outline: none;
}}
scale highlight {{
  background-color: {acct};
  min-height: 4px;
  border-radius: 999px;
  border: none;
  box-shadow: none;
}}
scale slider {{
  background-color: {txt1};
  border-radius: 999px;
  min-width: 16px;
  min-height: 16px;
  margin: -6px;
  border: none;
  box-shadow: 0 1px 3px alpha(black, 0.4);
  outline: none;
}}
scale slider:hover {{ background-color: {acct}; }}
scale:disabled {{ opacity: 0.35; }}

dropdown {{ background-color: {bgd3}; color: {txt1}; border-radius: 8px; }}

popover.menu contents {{ background-color: {bgd2}; border: 1px solid {bgd4}; border-radius: 8px; }}
popover.menu modelbutton {{ color: {txt1}; padding: 8px 12px; }}
popover.menu modelbutton:hover {{ background-color: {bgd3}; }}

.disconnected-scrim {{ background-color: alpha(black, 0.8); }}
.disconnected-card {{ background-color: {bgd1}; border-radius: 16px; }}
"#,
        bgd0 = p.bgd0,
        bgd1 = p.bgd1,
        bgd2 = p.bgd2,
        bgd3 = p.bgd3,
        bgd4 = p.bgd4,
        txt1 = p.txt1,
        txt2 = p.txt2,
        acct = p.acct,
        srch = p.srch,
        btns = p.btns,
    )
}
