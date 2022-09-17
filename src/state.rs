use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
};

use eframe::epaint::ahash::HashSet;

use egui::Vec2;
use egui_extras::RetainedImage;
use poll_promise::Promise;
use uuid::Uuid;

use crate::{
    helix,
    store::Image,
    twitch,
    widgets::{
        state::{self, ChatViewState},
        MainView, Position,
    },
    Channel, EnvConfig, FetchQueue, ImageCache, Interaction, KeyMapping, Queue, RequestPaint,
    UserListUpdater,
};

#[derive(Default)]
pub struct ViewState {
    pub current_view: MainView,
    pub previous_view: MainView,
}

impl ViewState {
    pub fn switch_to_view(&mut self, view: MainView) {
        if self.current_view == view {
            return;
        }
        self.previous_view = std::mem::replace(&mut self.current_view, view);
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.current_view, &mut self.previous_view)
    }
}

#[derive(Default)]
pub struct State {
    pub config: EnvConfig,
    pub key_mapping: KeyMapping,
    pub pixels_per_point: f32,

    pub channels: Vec<Channel>,

    pub chat_view_state: state::ChatViewState,

    pub settings: state::SettingsState,
    pub twitch_channels: state::TwitchChannelsState,
    pub twitch_settings: state::TwitchSettingsState,
    pub keybind_state: state::KeybindingsState,
    pub start_state: state::StartState,
    pub main_view: state::MainViewState,

    // TODO what is this
    pub view_state: ViewState,

    pub messages: Queue<twitch::Message>,

    pub emote_map: HashMap<String, String>,
    pub images: ImageCache,
    pub requested_images: HashSet<Uuid>,

    pub show_image_map: bool,

    pub window_size: Vec2,

    pub show_log_window: bool,
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
    pub helix: Promise<helix::Client>,
    pub fetch: FetchQueue<Image>,
    pub chatters_update: UserListUpdater,
    pub global_badges: Promise<Vec<helix::Badges>>,
    pub helix_ready: flume::Sender<helix::Client>,
}

pub struct AppState {
    pub twitch: Option<twitch::Twitch>,
    pub identity: Option<twitch::Identity>,
    pub interaction: Interaction,
    pub state: State,
    pub runtime: Runtime,

    pub dark_image_mask: RetainedImage,

    pub writer: flume::Sender<String>,
    pub reader: flume::Receiver<String>,
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

        for channel in self
            .state
            .channels
            .iter()
            .filter_map(|c| c.auto_join.then_some(&c.login))
        {
            self.join_channel(channel);
        }

        Ok(())
    }
}

impl AppState {
    pub fn new(
        repaint: impl RequestPaint + 'static,
        kappas: Vec<egui_extras::RetainedImage>,
        persist: PersistState,
        helix: Promise<helix::Client>,
        dark_image_mask: RetainedImage,
    ) -> Self {
        fn default<T: Default>() -> T {
            T::default()
        }

        let (helix_tx, helix_rx) = flume::bounded(0);
        let (writer, reader) = flume::unbounded();

        Self {
            state: State {
                pixels_per_point: persist.pixels_per_point,
                channels: persist.channels,
                config: persist.env_config,
                key_mapping: persist.key_mapping,
                chat_view_state: ChatViewState {
                    tab_bar_position: persist.tab_bar_position,
                    image_size: persist.tab_bar_image_size,
                    show_mask: persist.show_image_mask,
                    ..default()
                },
                start_state: state::StartState::new(kappas),
                ..default()
            },
            runtime: Runtime {
                helix,
                chatters_update: UserListUpdater::create(),
                fetch: FetchQueue::create(repaint),
                helix_ready: helix_tx,
                global_badges: Promise::spawn_thread("global_badges", {
                    move || {
                        let helix = helix_rx.recv().unwrap();
                        helix.get_badges().expect("get global badges")
                    }
                }),
            },
            twitch: default(),
            identity: default(),
            interaction: default(),

            dark_image_mask,

            writer,
            reader,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct PersistState {
    pub env_config: EnvConfig,
    pub key_mapping: KeyMapping,
    pub channels: Vec<Channel>,
    pub pixels_per_point: f32,
    pub tab_bar_position: Position,
    pub tab_bar_image_size: f32,
    pub show_image_mask: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BorrowedPersistState<'a> {
    pub env_config: &'a EnvConfig,
    pub key_mapping: &'a KeyMapping,
    pub channels: &'a Vec<Channel>,
    pub pixels_per_point: &'a f32,
    pub tab_bar_position: Position,
    pub tab_bar_image_size: f32,
    pub show_image_mask: bool,
}
