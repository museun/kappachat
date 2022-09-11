#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

use eframe::{NativeOptions, Storage};
use egui::{CentralPanel, Event, Key};

use kappachat::{
    helix, kappas,
    state::{AppState, BorrowedPersistState, PersistState},
    tabs, twitch,
    widgets::{self, MainView},
    CachedImages, EnvConfig, Interaction, KeyAction, TwitchLine,
};

pub struct App {
    context: egui::Context,
    interaction: Interaction,

    client: helix::Client,
    cached_images: CachedImages,

    app: AppState,
}

impl App {
    fn new(context: egui::Context, state: AppState) -> Self {
        Self {
            context,
            interaction: Interaction::create(),

            client: helix::Client::default(),
            cached_images: CachedImages::default(),

            app: state,
        }
    }

    fn connect(&mut self) -> anyhow::Result<()> {
        if self.app.twitch.is_some() {
            todo!("already connected")
        }

        let (client, identity) = {
            let reg = twitch::Registration {
                address: "irc.chat.twitch.tv:6667",
                nick: &self.app.state.config.twitch_name,
                pass: &self.app.state.config.twitch_oauth_token,
            };

            twitch::Client::connect(reg)
        }?;

        self.app.identity.replace(identity);
        self.app.twitch.replace(
            client.spawn_listen(self.context.clone()), //
        );

        // TODO display message saying that we're connected

        Ok(())
    }

    fn try_read(&mut self) -> bool {
        match &self.app.twitch {
            Some(twitch) => self
                .interaction
                .poll(twitch)
                .expect("FIXME: this should reset the state"), // XXX: what does this mean?
            _ => return false,
        }

        let msg = match self.interaction.try_read() {
            Some(item) => item,
            _ => return false,
        };

        use twitch::Command::*;
        match msg.command {
            Join => {
                let join = msg.as_join().expect("join message should be valid");
                if self.is_our_name(join.user) {
                    let chatters = self
                        .client
                        .get_chatters_for(join.channel.strip_prefix('#').unwrap_or(join.channel))
                        .unwrap();

                    self.app
                        .tabs
                        .get_mut(join.channel)
                        .update_chatters(chatters);
                    self.app.tabs.set_active_by_name(join.channel)
                }
            }

            Part => {
                let part = msg.as_part().expect("part message should be valid");
                if self.is_our_name(part.user) {
                    self.app.tabs.remove_tab(part.channel);
                }
            }

            Privmsg => {
                let pm = msg.as_privmsg().expect("privmsg message should be valid");
                let color = pm.color();
                let spans = vec![];

                // let spans = pm
                //     .emote_span()
                //     .into_iter()
                //     .map(|kind| match kind {
                //         twitch::TextKind::Text(inner) => {
                //             twitch::TextKind::Text(Cow::Owned(inner.to_string()))
                //         }
                //         twitch::TextKind::Emote(id) => twitch::TextKind::Emote(id),
                //     })
                //     .collect();

                let line = TwitchLine::new(
                    pm.sender, pm.target, //
                    pm.data, spans,
                )
                .with_color(color);
                self.app
                    .tabs
                    .get_mut(&line.source)
                    .append(tabs::Line::Twitch { line });
            }

            _ => {}
        }

        true
    }

    fn send_message(&self, target: &str, data: &str) {
        self.send_raw_fmt(format_args!("PRIVMSG {target} :{data}\r\n"))
    }

    fn join_channel(&self, channel: &str) {
        let octo = if !channel.starts_with('#') { "#" } else { "" };
        self.send_raw_fmt(format_args!("JOIN {octo}{channel}\r\n"))
    }

    fn part_channel(&self, channel: &str) {
        self.send_raw_fmt(format_args!("PART {channel}\r\n"))
    }

    fn send_raw_fmt(&self, raw: std::fmt::Arguments<'_>) {
        self.interaction.send_raw(raw);
    }

    fn identity(&self) -> &twitch::Identity {
        self.app.identity.as_ref().expect("initialization")
    }

    fn is_our_name(&self, name: &str) -> bool {
        self.identity().user_name == name
    }

    const fn is_connected(&self) -> bool {
        self.app.twitch.is_some()
    }

    fn toggle_tab_bar(&mut self) {
        self.app.showing_tab_bar = !self.app.showing_tab_bar;
    }

    fn toggle_user_list(&mut self) {
        self.app.tabs.active_mut().toggle_user_list()
    }

    fn toggle_line_mode(&mut self) {
        self.app.tabs.active_mut().next_line_mode()
    }

    fn toggle_timestamps(&mut self) {
        self.app.tabs.active_mut().toggle_timestamps()
    }

    fn next_tab(&mut self) {
        self.app.tabs.next_tab()
    }

    fn previous_tab(&mut self) {
        self.app.tabs.previous_tab()
    }

    fn try_set_active_tab(&mut self, index: usize) {
        self.app.tabs.set_active(index);
    }

    fn switch_to_settings(&mut self) {
        self.switch_to_view(widgets::MainViewState::Settings)
    }

    fn switch_to_main(&mut self) {
        self.switch_to_view(widgets::MainViewState::Main)
    }

    fn switch_to_view(&mut self, view: widgets::MainViewState) {
        if self.app.state.current_view == view {
            return;
        }
        self.app.state.previous_view = std::mem::replace(&mut self.app.state.current_view, view);
    }

    fn try_handle_key_press(&mut self) {
        if self.context.input().events.is_empty() {
            return;
        }

        if self.context.input().key_pressed(Key::F12) {
            self.context.set_debug_on_hover(
                !self.context.debug_on_hover(), //
            )
        }

        if self.app.state.keybind_state.is_capturing() {
            return;
        }

        let ctx = self.context.clone();
        for (key, modifiers) in ctx.input().events.iter().filter_map(|c| match c {
            &Event::Key {
                key,
                pressed,
                modifiers,
            } if !pressed => Some((key, modifiers)),
            _ => None,
        }) {
            if let Some(action) = self.app.state.key_mapping.find(key, modifiers) {
                use KeyAction::*;

                eprintln!("action: {action:?}");

                match action {
                    SwitchToSettings => self.switch_to_settings(),
                    SwitchToMain => self.switch_to_main(),

                    ToggleLineMode => self.toggle_line_mode(),
                    ToggleTabBar => self.toggle_tab_bar(),
                    ToggleTimestamps => self.toggle_timestamps(),
                    ToggleUserList => self.toggle_user_list(),

                    SwitchTab0 => self.try_set_active_tab(0),
                    SwitchTab1 => self.try_set_active_tab(1),
                    SwitchTab2 => self.try_set_active_tab(2),
                    SwitchTab3 => self.try_set_active_tab(3),
                    SwitchTab4 => self.try_set_active_tab(4),
                    SwitchTab5 => self.try_set_active_tab(5),
                    SwitchTab6 => self.try_set_active_tab(6),
                    SwitchTab7 => self.try_set_active_tab(7),
                    SwitchTab8 => self.try_set_active_tab(8),
                    SwitchTab9 => self.try_set_active_tab(9),

                    NextTab => self.next_tab(),
                    PreviousTab => self.previous_tab(),
                }
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.try_read();
        self.try_handle_key_press();

        CentralPanel::default().show(ctx, |ui| {
            if MainView::new(&mut self.app.state).display(ui) {
                // connect
            }
        });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let data = BorrowedPersistState {
            env_config: &self.app.state.config,
            key_mapping: &self.app.state.key_mapping,
            channels: &self.app.state.channels,
            pixels_per_point: &self.app.state.pixels_per_point,
        };

        let json = serde_json::to_string(&data).expect("valid json");
        storage.set_string(SETTINGS_KEY, json);
    }

    fn persist_native_window(&self) -> bool {
        true
    }
}

const SETTINGS_KEY: &str = "kappa_chat_settings";
const DEFAULT_PIXELS_PER_POINT: f32 = 1.0;

fn load_settings(state: &mut PersistState, storage: &dyn Storage) {
    let mut deser = match storage
        .get_string(SETTINGS_KEY)
        .as_deref()
        .map(serde_json::from_str::<PersistState>)
        .transpose()
        .ok()
        .flatten()
    {
        Some(deser) => deser,
        None => return,
    };

    state.pixels_per_point = deser.pixels_per_point;
    state.channels = deser.channels;
    state.key_mapping = deser.key_mapping;

    fn maybe_update<'a: 'e, 'b: 'e, 'e>(
        left: &'a mut EnvConfig,
        right: &'b mut EnvConfig,
        extract: fn(&'e mut EnvConfig) -> &'e mut String,
    ) {
        let left = extract(left);
        let right = extract(right);
        if left.trim().is_empty() {
            *left = std::mem::take(right)
        }
    }

    for extract in [
        (move |e| &mut e.twitch_oauth_token) as fn(&mut EnvConfig) -> &mut String,
        (move |e| &mut e.twitch_name) as fn(&mut EnvConfig) -> &mut String,
        (move |e| &mut e.twitch_client_id) as fn(&mut EnvConfig) -> &mut String,
        (move |e| &mut e.twitch_client_secret) as fn(&mut EnvConfig) -> &mut String,
    ] {
        maybe_update(&mut state.env_config, &mut deser.env_config, extract);
    }
}

fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from([".dev.env", ".secrets.env"]); //".secrets.env"

    // let mut cached = CachedImages::load_from("./data");

    // let json = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "badges.json"));
    // #[derive(serde::Deserialize)]
    // struct Resp<T> {
    //     data: Vec<T>,
    // }
    // let badges = serde_json::from_str::<Resp<helix::Badges>>(json)
    //     .unwrap()
    //     .data;

    // cached.merge_badges(&badges);

    let kappas = kappas::load_kappas();

    let mut state = PersistState {
        env_config: EnvConfig::load_from_env(),
        pixels_per_point: DEFAULT_PIXELS_PER_POINT,
        ..Default::default()
    };

    eframe::run_native(
        "KappaChat",
        NativeOptions::default(),
        Box::new(move |cc| {
            if let Some(storage) = cc.storage {
                load_settings(&mut state, storage);
            }

            cc.egui_ctx.set_pixels_per_point(state.pixels_per_point);

            let state = AppState::new(kappas, state);
            Box::new(App::new(cc.egui_ctx.clone(), state))
        }),
    );

    Ok(())
}
