use std::hash::{Hash, Hasher};

use crate::{
    widgets::{
        KeybindingsState, MainView, SettingsState, StartState, TwitchChannelsState,
        TwitchSettingsState,
    },
    Channel, EnvConfig, KeyMapping,
};

#[derive(Default)]
pub struct State {
    pub config: EnvConfig,
    pub channels: Vec<Channel>,
    pub settings: SettingsState,
    pub pixels_per_point: f32,
    pub key_mapping: KeyMapping,
    pub twitch_channels: TwitchChannelsState,
    pub twitch_settings: TwitchSettingsState,
    pub keybind_state: KeybindingsState,
    pub current_view: MainView,
    pub previous_view: MainView,
    pub start_state: StartState,
}

impl State {
    pub fn make_hash(input: impl Hash) -> u64 {
        use std::collections::hash_map::DefaultHasher as H;
        let mut state = H::new();
        input.hash(&mut state);
        state.finish()
    }
}
