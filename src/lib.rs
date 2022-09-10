#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

pub const TWITCH_COLOR: Color32 = Color32::from_rgb(146, 86, 237);

pub fn get_small_font_id(ui: &egui::Ui) -> egui::FontId {
    egui::TextStyle::Small.resolve(&*ui.style())
}
pub fn get_body_font_id(ui: &egui::Ui) -> egui::FontId {
    egui::TextStyle::Body.resolve(&*ui.style())
}
pub fn get_monospace_font_id(ui: &egui::Ui) -> egui::FontId {
    egui::TextStyle::Monospace.resolve(&*ui.style())
}
pub fn get_button_font_id(ui: &egui::Ui) -> egui::FontId {
    egui::TextStyle::Button.resolve(&*ui.style())
}
pub fn get_heading_font_id(ui: &egui::Ui) -> egui::FontId {
    egui::TextStyle::Heading.resolve(&*ui.style())
}

pub trait RequestPaint: Send + Sync {
    fn request_repaint(&self) {}
}

impl RequestPaint for egui::Context {
    fn request_repaint(&self) {
        Self::request_repaint(self)
    }
}

pub struct NoopRepaint;
impl RequestPaint for NoopRepaint {}

mod state;
use egui::Color32;
pub use state::SettingsState;

pub mod widgets;

mod action;

mod command;
pub use command::Command;

mod config;
pub use config::EnvConfig;

mod key_mapping;
pub use key_mapping::{Chord, KeyAction, KeyHelper, KeyMapping};

pub mod helix;
pub use helix::CachedImages;

pub mod tabs;
pub use tabs::{Line, Tabs};

mod line;
pub use line::TwitchLine;

mod chat_layout;
pub use chat_layout::ChatLayout;

mod queue;
pub use queue::Queue;

pub mod twitch;

mod ext;
pub use ext::JobExt as _;

mod interaction;
pub use interaction::Interaction;

pub mod kappas;

pub mod font_icon {
    pub const HIDDEN: &str = "👁";
    pub const ADD: &str = "➕";
    pub const REMOVE: &str = "✖";
    pub const UNDO: &str = "🔄";
    pub const HELP: &str = "❔";
    pub const TIME: &str = "⏰";
    pub const AUTOJOIN: &str = "🔜";
    pub const USER_LIST: &str = "🚮";
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct PersistState {
    pub env_config: EnvConfig,
    pub key_mapping: KeyMapping,
    pub channels: Vec<Channel>,
    pub pixels_per_point: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BorrowedPersistState<'a> {
    pub env_config: &'a EnvConfig,
    pub key_mapping: &'a KeyMapping,
    pub channels: &'a Vec<Channel>,
    pub pixels_per_point: &'a f32,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Channel {
    pub name: String,
    pub show_timestamps: bool,
    pub show_user_list: bool,
    pub auto_join: bool,
}

impl Channel {
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            show_timestamps: true,
            show_user_list: true,
            auto_join: true,
        }
    }

    pub fn temporary(self) -> Self {
        Self {
            auto_join: false,
            ..self
        }
    }
}

pub struct State {
    pub twitch: Option<twitch::Twitch>,
    pub identity: Option<twitch::Identity>,
    pub settings_state: SettingsState,

    pub config: EnvConfig,
    pub key_mapping: KeyMapping,

    pub line: Option<String>,

    pub last: std::time::Instant,
    pub kappa_index: usize,
    pub kappas: [egui_extras::RetainedImage; 5],

    pub start_rotation: widgets::StartRotation,

    pub scroll: f32,

    pub tabs: Tabs,
    pub showing_tab_bar: bool,
    pub showing_help: widgets::HelpView,
}

impl State {
    pub fn new(kappas: [egui_extras::RetainedImage; 5], persist: PersistState) -> Self {
        Self {
            twitch: None,
            identity: None,
            settings_state: SettingsState {
                pixels_per_point: persist.pixels_per_point,
                channels: persist.channels,
                ..SettingsState::default()
            },
            config: persist.env_config,
            key_mapping: persist.key_mapping,
            line: None,
            last: std::time::Instant::now(),
            kappa_index: 0,
            kappas,
            start_rotation: widgets::StartRotation::new(),
            scroll: 0.0,
            tabs: Tabs::create(),
            showing_tab_bar: false,
            showing_help: widgets::HelpView::default(),
        }
    }
}
