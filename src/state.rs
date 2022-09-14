use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use egui_extras::RetainedImage;

use crate::{
    helix,
    twitch::{self, EmoteSpan},
    widgets::{
        settings::KeybindingsState, settings::TwitchChannelsState, settings::TwitchSettingsState,
        ChatViewState, MainViewState, MainViewView, SettingsState, StartState,
    },
    Channel, EnvConfig, FetchQueue, Interaction, KeyMapping, Queue, RequestPaint, TwitchImage,
};

#[derive(Default)]
pub struct ViewState {
    pub current_view: MainViewView,
    pub previous_view: MainViewView,
}

impl ViewState {
    pub fn switch_to_view(&mut self, view: MainViewView) {
        if self.current_view == view {
            return;
        }
        self.previous_view = std::mem::replace(&mut self.current_view, view);
    }
}

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
    pub start_state: StartState,
    pub chat_view_state: ChatViewState,

    pub view_state: ViewState,

    pub main_view: MainViewState,
    pub messages: Queue<twitch::Message>,

    pub spanned_lines: HashMap<uuid::Uuid, Vec<EmoteSpan>>,

    pub emote_map: HashMap<String, String>,
    pub images: HashMap<String, RetainedImage>,
}

impl State {
    pub fn make_hash(input: impl Hash) -> u64 {
        use std::collections::hash_map::DefaultHasher as H;
        let mut state = H::new();
        input.hash(&mut state);
        state.finish()
    }
}

pub struct Runtime {
    pub helix: poll_promise::Promise<helix::Client>,
    pub fetch: FetchQueue<TwitchImage>,
}

pub struct AppState {
    pub twitch: Option<twitch::Twitch>,
    pub identity: Option<twitch::Identity>,
    pub interaction: Interaction,
    pub state: State,
    pub runtime: Runtime,
}

impl AppState {
    pub fn identity(&self) -> &twitch::Identity {
        self.identity.as_ref().expect("initialization")
    }

    pub fn is_our_name(&self, name: &str) -> bool {
        self.identity().user_name == name
    }

    pub fn send_message(&self, target: &str, data: &str) {
        self.send_raw_fmt(format_args!("PRIVMSG {target} :{data}\r\n"))
    }

    pub fn join_channel(&self, channel: &str) {
        let octo = if !channel.starts_with('#') { "#" } else { "" };
        self.send_raw_fmt(format_args!("JOIN {octo}{channel}\r\n"))
    }

    pub fn part_channel(&self, channel: &str) {
        self.send_raw_fmt(format_args!("PART {channel}\r\n"))
    }

    fn send_raw_fmt(&self, raw: std::fmt::Arguments<'_>) {
        self.interaction.send_raw(raw);
    }

    pub fn connect(&mut self, painter: impl RequestPaint + 'static) -> anyhow::Result<()> {
        if self.twitch.is_some() {
            todo!("already connected")
        }

        let (client, identity) = {
            let reg = twitch::Registration {
                address: "irc.chat.twitch.tv:6667",
                nick: &self.state.config.twitch_name,
                pass: &self.state.config.twitch_oauth_token,
            };

            twitch::Client::connect(reg)
        }?;

        self.identity.replace(identity);
        self.twitch.replace(client.spawn_listen(painter));

        Ok(())
    }
}

impl AppState {
    pub fn new(
        repaint: impl RequestPaint + 'static,
        kappas: Vec<egui_extras::RetainedImage>,
        persist: PersistState,
        helix: poll_promise::Promise<helix::Client>,
    ) -> Self {
        Self {
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
            runtime: Runtime {
                helix,
                fetch: FetchQueue::new(repaint),
            },
            twitch: Default::default(),
            identity: Default::default(),
            interaction: Default::default(),
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
