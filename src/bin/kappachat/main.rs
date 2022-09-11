use eframe::{NativeOptions, Storage};
use egui::{text::LayoutJob, CentralPanel, Color32, Event, Key, TextFormat, TextStyle};

use kappachat::{
    helix, kappas, tabs, twitch,
    widgets::{self, MainWidget},
    AppState, BorrowedPersistState, CachedImages, Command, EnvConfig, Interaction, KeyAction, Line,
    PersistState, TwitchLine,
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

    const fn verify_config(&self) -> bool {
        let EnvConfig {
            twitch_name: _twitch_name,
            twitch_oauth_token: _twitch_oauth_token,
            twitch_client_id: _twitch_client_id,
            twitch_client_secret: _twitch_client_secret,
        } = &self.app.state.config;

        // if twitch_name.is_some()
        //     & twitch_oauth_token.is_some()
        //     & twitch_client_id.is_some()
        //     & twitch_client_secret.is_some()
        // {
        //     return true;
        // }

        // TODO use the validators from the settings?

        false
    }

    fn connect(&mut self) -> anyhow::Result<()> {
        if !self.verify_config() {
            self.switch_to_settings();
            return Ok(());
        }

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
                .expect("FIXME: this should reset the state"),
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

    fn report_error(&mut self, error: Line) {
        self.app.tabs.get_mut("*status").append(error);
    }

    fn create_error(&mut self, prefix: impl ToString, msg: impl AsRef<str>) {
        let id = TextStyle::Body.resolve(&*self.context.style());
        let mut layout = LayoutJob::simple_singleline(prefix.to_string(), id.clone(), Color32::RED);
        layout.append(msg.as_ref(), 5.0, TextFormat::simple(id, Color32::GRAY));

        let msg = layout;
        self.report_error(Line::Status { msg })
    }

    fn check_if_connected(&mut self, cmd: &Command<'_>) -> bool {
        if self.is_connected() || matches!(cmd, Command::Connect) {
            return true;
        }

        self.create_error("not connected:", &cmd.report());
        false
    }

    fn send_line(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }

        // TODO get rid of all of this
        let cmd = Command::parse(line);
        if !self.check_if_connected(&cmd) {
            return;
        }

        match cmd {
            Command::Message { raw } => {
                let target = &self.app.tabs.active().title;
                self.send_message(target, raw);

                let twitch::Identity {
                    user_name, color, ..
                } = self.identity();

                let line = TwitchLine::new(
                    user_name, //
                    target,
                    raw,
                    vec![],
                )
                .with_color(color);

                self.app
                    .tabs
                    .active_mut()
                    .append(tabs::Line::Twitch { line });
            }

            Command::Join { channel } => {
                for channel in channel.split(',') {
                    self.join_channel(channel);
                }
            }

            Command::Part { channel } => {
                let target = channel.unwrap_or_else(|| &self.app.tabs.active().title);
                self.part_channel(target);
            }

            Command::Connect => {
                if let Err(err) = self.connect() {
                    // TODO Line::Broadcast
                    self.create_error("disconnected", err.to_string());
                }
            }

            Command::Invalid { raw } => {
                self.create_error("invalid command:", raw);
            }

            Command::Nothing => {}
        }
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
        if matches!(self.app.state.current_view, widgets::MainView::Settings) {
            return;
        }

        self.app.state.previous_view = std::mem::replace(
            &mut self.app.state.current_view,
            widgets::MainView::Settings,
        );
    }

    fn switch_to_main(&mut self) {
        if matches!(self.app.state.current_view, widgets::MainView::Main) {
            return;
        }

        self.app.state.previous_view =
            std::mem::replace(&mut self.app.state.current_view, widgets::MainView::Main);
    }

    fn try_handle_key_press(&mut self) {
        if self.context.input().events.is_empty() {
            return;
        }

        if self.context.input().key_pressed(Key::F12) {
            self.context
                .set_debug_on_hover(!self.context.debug_on_hover())
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
                eprintln!("action: {action:?}");

                use KeyAction::*;
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

impl App {
    fn try_display_tab_bar(&mut self) {
        if self.app.showing_tab_bar {
            egui::panel::TopBottomPanel::top("top")
                .resizable(false)
                .show(&self.context, |ui| ui.add(&mut self.app.tabs));
        }
    }

    fn try_send_line(&mut self) {
        if let Some(line) = &mut self.app.line {
            if !line.is_empty() {
                let line = std::mem::take(line);
                self.send_line(&line);
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.try_read();

        self.try_handle_key_press();

        CentralPanel::default().show(ctx, |ui| {
            MainWidget::new(&mut self.app.state).display(ui);
        });

        // self.try_display_help();

        // if !self.try_display_start_screen() {
        //     return;
        // }

        // egui::panel::TopBottomPanel::bottom("bottom")
        //     .resizable(false)
        //     .frame(egui::Frame::none().fill(Color32::BLACK))
        //     .show(ctx, |ui| {
        //         widgets::EditBox::new(&mut self.state.tabs, &mut self.state.line).ui(ui)
        //     });

        // self.try_send_line();

        // self.try_display_tab_bar();

        // // TODO redo this
        // let pos = self.state.scroll;
        // self.state.scroll += ctx.input().scroll_delta.y;

        // egui::panel::CentralPanel::default().show(ctx, |ui| {
        //     widgets::TabWidget {
        //         tab: self.state.tabs.active_mut(),
        //         cached_images: &mut self.cached_images,
        //         stick: pos != self.state.scroll,
        //     }
        //     .ui(ui)
        // });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let data = BorrowedPersistState {
            env_config: &self.app.state.config,
            key_mapping: &self.app.state.key_mapping,
            channels: &self.app.state.channels,
            pixels_per_point: &self.app.state.pixels_per_point,
        };

        let s = serde_json::to_string(&data).expect("valid json");
        storage.set_string(SETTINGS_KEY, s);
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
    simple_env_load::load_env_from([".dev.env"]); //".secrets.env"

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
