use std::collections::HashMap;

use egui::Id;

use crate::command::Command;

#[derive(Default)]
pub struct SettingsState {
    pub pixels_per_point: f32,
    pub channels: Vec<String>,
    pub channels_to_remove: Vec<String>,
    pub adding_channel_id: Option<Id>,
    pub adding_channel: Option<String>,
    pub twitch_visible: HashMap<u64, bool>,

    pub command: Option<Command<'static>>,
}

impl SettingsState {
    pub fn make_hash(input: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash as _, Hasher as _};
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        hasher.finish()
    }

    pub fn dpi_repr(f: f32) -> &'static str {
        const LOOKUP: [&str; 11] = [
            "1.0", "1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7", "1.8", "1.9", "2.0",
        ];
        let index = ((f * 10.0) as usize) - 10;
        LOOKUP[index]
    }

    pub fn dpi_range() -> impl Iterator<Item = f32> {
        std::iter::successors(Some(1.0_f32), |a| Some(a + 0.1)).take(11)
    }
}
