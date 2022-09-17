#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

// TODO log view

use egui::Color32;

pub const TWITCH_COLOR: Color32 = Color32::from_rgb(146, 86, 237);
pub const SETTINGS_KEY: &str = "kappa_chat_settings";
pub const APP_NAME: &str = "KappaChat";

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

pub mod app;
mod channel;
mod config;
mod fetch;
pub mod font_icon;
pub mod helix;
mod image_cache;
mod interaction;
pub mod kappas;
mod key_mapping;
mod queue;
pub mod state;
mod task_queue;
pub mod twitch;
mod user_list_updater;
pub mod widgets;

pub use app::App;
pub use channel::Channel;
pub use config::EnvConfig;
pub use fetch::{FetchImage, FetchQueue};
pub use image_cache::ImageCache;
pub use interaction::Interaction;
pub use key_mapping::{Chord, KeyAction, KeyHelper, KeyMapping};
pub use queue::Queue;
pub use task_queue::TaskQueue;
use user_list_updater::UserListUpdater;

mod store;

// TODO make a light version of this mask
pub const DARK_MASK_PNG: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/mask.png"));

pub(crate) static FORMAT: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[hour]:[minute]:[second]");

pub fn format_seconds(mut secs: u64) -> String {
    const TABLE: [(&str, u64); 4] = [
        ("days", 86400),
        ("hours", 3600),
        ("minutes", 60),
        ("seconds", 1),
    ];

    fn pluralize(s: &str, n: u64) -> String {
        format!("{} {}", n, if n > 1 { s } else { &s[..s.len() - 1] })
    }

    let mut time = vec![];
    for (name, d) in &TABLE {
        let div = secs / d;
        if div > 0 {
            time.push(pluralize(name, div));
            secs -= d * div;
        }
    }

    let len = time.len();
    if len > 1 {
        if len > 2 {
            for segment in time.iter_mut().take(len - 2) {
                segment.push(',')
            }
        }
        time.insert(len - 1, "and".into())
    }
    time.join(" ")
}

mod logger;
pub use logger::init_logger;
