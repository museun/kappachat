#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

use egui::Color32;

pub const TWITCH_COLOR: Color32 = Color32::from_rgb(146, 86, 237);

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

pub mod state;

pub mod widgets;

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

pub mod font_icon;

mod channel;
pub use channel::Channel;
