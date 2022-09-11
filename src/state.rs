use std::hash::{Hash, Hasher};

use crate::{
    twitch,
    widgets::{
        KeybindingsState, MainView, SettingsState, StartState, TwitchChannelsState,
        TwitchSettingsState,
    },
    Channel, EnvConfig, KeyMapping, Tabs,
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

pub struct AppState {
    pub twitch: Option<twitch::Twitch>,
    pub identity: Option<twitch::Identity>,
    pub state: State,

    pub line: Option<String>,

    pub scroll: f32,

    pub tabs: Tabs,
    pub showing_tab_bar: bool,
}

impl AppState {
    // TODO redo this
    pub fn new(kappas: Vec<egui_extras::RetainedImage>, persist: PersistState) -> Self {
        Self {
            twitch: None,
            identity: None,
            state: State {
                pixels_per_point: persist.pixels_per_point,
                channels: persist.channels,
                config: persist.env_config,
                key_mapping: persist.key_mapping,
                start_state: StartState {
                    kappas,
                    ..Default::default()
                },
                ..Default::default()
            },
            line: None,
            scroll: 0.0,
            tabs: Tabs::create(),
            showing_tab_bar: false,
        }
    }
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
