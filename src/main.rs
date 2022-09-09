#![cfg_attr(debug_assertions, allow(dead_code, unused_variables,))]

use std::time::Instant;

use eframe::NativeOptions;
use egui::{
    text::LayoutJob, CentralPanel, Color32, Event, Key, TextEdit, TextFormat, TextStyle, Widget,
};
use egui_extras::RetainedImage;

struct App {
    interaction: Interaction,
    context: egui::Context,

    twitch: Option<twitch::Twitch>,
    identity: Option<twitch::Identity>,

    config: EnvConfig,
    key_mapping: KeyMapping,

    client: helix::Client,
    cached_images: CachedImages,

    tabs: Tabs,
    showing_tab_bar: bool,
    showing_help: widgets::HelpView,
    scroll: f32,

    settings_state: SettingsState,

    last: Instant,
    kappa_index: usize,

    kappas: [RetainedImage; 5],
}

impl App {
    fn new(
        context: egui::Context,
        config: EnvConfig,
        key_mapping: KeyMapping,
        client: helix::Client,
        cached_images: CachedImages,
        pixels_per_point: f32,

        kappas: [RetainedImage; 5],
    ) -> Self {
        Self {
            interaction: Interaction::create(),
            context,

            twitch: None,
            identity: None,

            config,
            key_mapping,

            client,
            cached_images,

            tabs: Tabs::create(),
            showing_tab_bar: true,
            showing_help: widgets::HelpView::None,
            scroll: 0.0,

            settings_state: SettingsState {
                pixels_per_point,
                ..SettingsState::default()
            },

            kappas,
            kappa_index: 0,

            last: Instant::now(),
        }
    }

    fn connect(&mut self) -> anyhow::Result<()> {
        if self.twitch.is_some() {
            todo!("already connected")
        }

        let (client, identity) = {
            let reg = twitch::Registration {
                address: "irc.chat.twitch.tv:6667",
                nick: &self.config.twitch_name,
                pass: &self.config.twitch_oauth_token,
            };

            twitch::Client::connect(reg)
        }?;

        self.identity.replace(identity);
        self.twitch.replace(
            client.spawn_listen(self.context.clone()), //
        );

        // TODO display message saying that we're connected

        Ok(())
    }

    fn try_read(&mut self, ctx: &egui::Context) -> bool {
        match &self.twitch {
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

                    self.tabs.get_mut(join.channel).update_chatters(chatters);
                    self.tabs.set_active_by_name(join.channel)
                }
            }

            Part => {
                let part = msg.as_part().expect("part message should be valid");
                if self.is_our_name(part.user) {
                    self.tabs.remove_tab(part.channel);
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
                self.tabs
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
        self.identity.as_ref().expect("initialization")
    }

    fn is_our_name(&self, name: &str) -> bool {
        self.identity().user_name == name
    }

    const fn is_connected(&self) -> bool {
        self.twitch.is_some()
    }

    fn report_error(&mut self, error: Line) {
        self.tabs.get_mut("*status").append(error);
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

        let cmd = Command::parse(line);
        if !self.check_if_connected(&cmd) {
            return;
        }

        match cmd {
            Command::Message { raw } => {
                let target = &self.tabs.active().title();
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

                self.tabs.active_mut().append(tabs::Line::Twitch { line });
            }

            Command::Join { channel } => {
                for channel in channel.split(',') {
                    self.join_channel(channel);
                }
            }

            Command::Part { channel } => {
                let target = channel.unwrap_or_else(|| self.tabs.active().title());
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
        self.showing_tab_bar = !self.showing_tab_bar;
    }

    fn toggle_user_list(&mut self) {
        self.tabs.active_mut().toggle_user_list()
    }

    fn toggle_line_mode(&mut self) {
        self.tabs.active_mut().next_line_mode();
    }

    fn toggle_timestamps(&mut self) {
        self.tabs.active_mut().toggle_timestamps()
    }

    fn next_tab(&mut self) {
        self.tabs.next_tab()
    }

    fn previous_tab(&mut self) {
        self.tabs.previous_tab()
    }

    fn toggle_help(&mut self) {
        self.showing_help = if matches!(self.showing_help, widgets::HelpView::None) {
            widgets::HelpView::KeyBindings
        } else {
            widgets::HelpView::None
        }
    }

    fn try_set_active_tab(&mut self, index: usize) {
        self.tabs.set_active(index);
    }

    fn try_handle_key_press(&mut self) {
        let ctx = self.context.clone();

        for (key, modifiers) in ctx.input().events.iter().filter_map(|c| match c {
            &Event::Key {
                key,
                pressed,
                modifiers,
            } if !pressed => Some((key, modifiers)),
            _ => None,
        }) {
            if let Some(action) = self.key_mapping.find(key, modifiers) {
                eprintln!("action: {action:?}");

                use KeyAction::*;
                match action {
                    ToggleHelp => self.toggle_help(),
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

mod state;
use state::SettingsState;

mod widgets;

impl App {
    fn try_display_help(&mut self, ctx: &egui::Context) {
        if !matches!(self.showing_help, widgets::HelpView::None) {
            egui::Window::new("Help")
                .title_bar(false)
                .resizable(false)
                .vscroll(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    widgets::Help::new(
                        &mut self.showing_help,
                        &mut self.key_mapping,
                        &mut self.settings_state,
                        &mut self.showing_tab_bar,
                        &mut self.tabs,
                        &mut self.config,
                    )
                    .ui(ui)
                });
        }
    }

    fn try_display_tab_bar(&mut self, ctx: &egui::Context) {
        if self.showing_tab_bar {
            egui::panel::TopBottomPanel::top("top")
                .resizable(false)
                .show(ctx, |ui| ui.add(&mut self.tabs));
        }
    }

    fn display_edit_box(&mut self, ctx: &egui::Context) {
        egui::panel::TopBottomPanel::bottom("bottom")
            .resizable(false)
            .frame(egui::Frame::none().fill(Color32::BLACK))
            .show(ctx, |ui| {
                // TODO multi-line edit box
                let resp = ui.add(
                    TextEdit::singleline(self.tabs.active_mut().buffer_mut())
                        .frame(false)
                        .lock_focus(true),
                );

                let id = resp.id;
                if resp.lost_focus() && ctx.input().key_pressed(Key::Enter) {
                    let input = std::mem::take(self.tabs.active_mut().buffer_mut());
                    self.send_line(&input);
                }

                ctx.memory().request_focus(id);
            });
    }

    fn try_display_start_screen(&mut self, ctx: &egui::Context) -> bool {
        if self.is_connected() {
            return true;
        }

        CentralPanel::default().show(ctx, |ui| {
            widgets::StartScreen {
                images: &self.kappas,
                last: &mut self.last,
                index: &mut self.kappa_index,
                command: &mut self.settings_state.command,
            }
            .ui(ui)
        });

        if let Some(Command::Connect) = &self.settings_state.command {
            self.settings_state.command.take();
            self.connect().expect("connect") // TODO handle this
        };

        false
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.try_read(ctx);

        self.try_handle_key_press();
        self.try_display_help(ctx);

        if !self.try_display_start_screen(ctx) {
            return;
        }

        self.display_edit_box(&ctx);
        self.try_display_tab_bar(ctx);

        // TODO redo this
        let pos = self.scroll;
        self.scroll += ctx.input().scroll_delta.y;

        egui::panel::CentralPanel::default().show(ctx, |ui| {
            widgets::TabWidget {
                tab: self.tabs.active_mut(),
                cached_images: &mut self.cached_images,
                stick: pos != self.scroll,
            }
            .ui(ui)
        });
    }

    fn persist_native_window(&self) -> bool {
        true
    }
}

pub trait RequestPaint: Send + Sync {
    fn request_repaint(&self) {}
}

impl RequestPaint for egui::Context {
    fn request_repaint(&self) {
        egui::Context::request_repaint(self)
    }
}

pub struct NoopRepaint;
impl RequestPaint for NoopRepaint {}

mod action;

mod command;
use command::Command;

mod config;
pub use config::EnvConfig;

mod key_mapping;
pub use key_mapping::{Chord, KeyAction, KeyHelper, KeyMapping};

mod helix;
use helix::CachedImages;

mod tabs;
use tabs::{Line, Tabs};

mod line;
use line::TwitchLine;

mod chat_layout;
use chat_layout::ChatLayout;

mod queue;
use queue::Queue;

mod twitch;

mod ext;
pub use ext::JobExt as _;

mod interaction;
pub use interaction::Interaction;

mod kappas;

fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from([".dev.env", ".secrets.env"]);

    let env_config = EnvConfig::load_from_env()?;

    // TODO this should be done in the background, it does an http request
    let client = helix::Client::fake(
        &env_config.twitch_client_id,
        &env_config.twitch_client_secret,
    )?;

    // let mut cached = CachedImages::load_from("./data");

    let cached = CachedImages::default();

    // let json = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", "badges.json"));
    // #[derive(serde::Deserialize)]
    // struct Resp<T> {
    //     data: Vec<T>,
    // }
    // let badges = serde_json::from_str::<Resp<helix::Badges>>(json)
    //     .unwrap()
    //     .data;

    // cached.merge_badges(&badges);

    let mut key_mapping = None;

    let mut pixels_per_point = 1.0;

    let kappas = kappas::load_kappas();

    eframe::run_native(
        "KappaChat",
        NativeOptions {
            ..Default::default()
        },
        Box::new(move |cc| {
            if let Some(storage) = cc.storage {
                if let Some(ppp) = storage
                    .get_string("window_pixels_per_point")
                    .and_then(|s| s.parse().ok())
                {
                    eprintln!("setting pixels per point: {ppp:.1}");
                    cc.egui_ctx.set_pixels_per_point(ppp);
                    pixels_per_point = ppp
                }

                if let Some(keys) = storage
                    .get_string("keybindings")
                    .and_then(|s| serde_yaml::from_str(&s).ok())
                {
                    key_mapping.replace(keys);
                }
            }

            Box::new(App::new(
                cc.egui_ctx.clone(),
                env_config,
                key_mapping.unwrap_or_default(),
                client,
                cached,
                pixels_per_point,
                kappas,
            ))
        }),
    );

    Ok(())
}
