//! Peek: hold ⌥ over a Haunt row to see that agent's terminal in a floating card; let go and it's
//! gone, with nothing switched.
// wit-bindgen's flattened `notify` import has 14 params.
#![allow(clippy::too_many_arguments)]

wit_bindgen::generate!({ path: "wit/haunt-plugin.wit", world: "plugin" });

// Shared with the other fleet-following plugins; not every helper is used here.
#[allow(dead_code)]
mod fleet;
mod peek;

use std::sync::Mutex;

use fleet::{Ev, Fleet};
use haunt::plugin::log::log;
use haunt::plugin::panel::{self, Node, PanelNode, Placement, TextNode, Tone};
use haunt::plugin::types::LogLevel;
use haunt::plugin::{keys, terminal};
use serde::Deserialize;

const CARD: &str = "peek";
/// The card's width in characters.
const COLS: usize = 80;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Lines in the card, 1 to 50.
    pub lines: u32,
    pub unverified: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self { lines: 8, unverified: false }
    }
}

#[derive(Default)]
struct Live {
    settings: Settings,
    fleet: Fleet,
}

static LIVE: Mutex<Option<Live>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut Live) -> R) -> R {
    f(LIVE.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(Live::default))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

/// The card: the screen's last lines in mono, or a dim line when there's nothing to show.
pub fn card(lines: &[String]) -> Vec<PanelNode> {
    let text = |text: String, tone| PanelNode { parent: None, node: Node::Text(TextNode { text, tone, mono: true, bold: false }) };
    if lines.is_empty() {
        return vec![text("nothing on screen yet".into(), Tone::Dim)];
    }
    lines.iter().map(|l| text(l.clone(), Tone::Text)).collect()
}

fn open(l: &Live, surface: &str) {
    let rows = l.settings.lines.clamp(1, 50);
    let screen = match terminal::read(surface, rows) {
        Ok(lines) => lines.join("\n"),
        Err(e) => return log(LogLevel::Warn, &format!("peek: screen not read: {e}")),
    };
    let name = l.fleet.agents.get(surface).map_or("terminal", |a| a.name.as_str());
    if let Err(e) = panel::show(CARD, Placement::Card, name, &card(&peek::tail(&screen, rows as usize, COLS))) {
        log(LogLevel::Warn, &format!("peek: no card: {e}"));
    }
}

struct Plugin;

impl Guest for Plugin {
    fn init(json: String) {
        with(|l| l.settings = serde_json::from_str(&json).unwrap_or_default());
        if let Err(e) = keys::hover_hold(Some("alt")) {
            log(LogLevel::Warn, &format!("peek: ⌥ hover not taken: {e}"));
        }
    }

    fn on_settings_changed(json: String) {
        with(|l| l.settings = serde_json::from_str(&json).unwrap_or_default());
    }

    fn on_event(event: AgentEvent) {
        let Some(surface) = event.surface.as_deref() else { return };
        with(|l| {
            if !event.verified && !l.settings.unverified {
                return;
            }
            let ev = Ev { surface, kind: &event.kind, cwd: event.cwd.as_deref(), summary: event.summary.as_deref(), subagent: event.agent_id.is_some() };
            l.fleet.apply(&ev, now_ms());
        });
    }

    fn palette_items(_mode: String, _query: String) -> Vec<PaletteItem> {
        Vec::new()
    }

    fn on_action(_action_id: String, _payload: String) {}

    fn row_menu(_surface: String) -> Vec<MenuItem> {
        Vec::new()
    }

    fn on_row_moved(_surface: String, _group: String, _before: Option<String>) {}

    fn on_theme_changed() {}

    fn on_tick(_now_ms: u64) {}

    fn on_exec(_job: u64, _outcome: Result<haunt::plugin::exec::Output, String>) {}

    fn on_away_changed(_away: bool, _away_ms: u64) {}

    fn on_hover_hold(held: bool, surface: String) {
        if held {
            with(|l| open(l, &surface));
        } else {
            panel::close(CARD);
        }
    }

    fn on_fs_tree(_job: u64, _outcome: Result<haunt::plugin::fs_list::TreeResult, String>) {}
}

export!(Plugin);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_card_is_the_screen_in_mono() {
        let n = card(&["$ npm test session".into(), " RUNS  refresh.test.ts".into()]);
        assert!(matches!(&n[1].node, Node::Text(t) if t.mono && t.text == " RUNS  refresh.test.ts" && matches!(t.tone, Tone::Text)));
        assert!(matches!(&card(&[])[0].node, Node::Text(t) if matches!(t.tone, Tone::Dim)));
    }

    #[test]
    fn manifest_asks_for_what_it_uses_and_defaults_match() {
        let m = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/haunt-plugin.toml")).unwrap();
        let t: toml::Table = toml::from_str(&m).unwrap();
        let caps = t["capabilities"].as_table().unwrap();
        assert_eq!(caps.keys().collect::<Vec<_>>(), ["events", "keys", "panel", "terminal_read"]);
        let defaults: serde_json::Map<String, serde_json::Value> =
            t["settings"].as_table().unwrap().iter().map(|(k, v)| (k.clone(), serde_json::to_value(&v["default"]).unwrap())).collect();
        assert_eq!(serde_json::from_value::<Settings>(serde_json::Value::Object(defaults)).unwrap(), Settings::default());
    }
}
