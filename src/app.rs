use crate::{
    helix,
    state::{AppState, BorrowedPersistState},
    widgets::{MainView, MainViewView},
    CachedImages, SETTINGS_KEY,
};

pub struct App {
    context: egui::Context,

    client: helix::Client,
    cached_images: CachedImages,

    pub app: AppState,
}

impl App {
    pub fn new(context: egui::Context, state: AppState) -> Self {
        Self {
            context,

            client: helix::Client::default(),
            cached_images: CachedImages::default(),

            app: state,
        }
    }

    fn try_read(&mut self) -> bool {
        match &self.app.twitch {
            Some(twitch) => self
                .app
                .interaction
                .poll(twitch)
                .expect("FIXME: this should reset the state"), // XXX: what does this mean?
            _ => return false,
        }

        let msg = match self.app.interaction.try_read() {
            Some(item) => item,
            _ => return false,
        };

        self.app.state.messages.push(msg);

        // use twitch::Command::*;
        // match msg.command {
        //     Join => {
        //         let join = msg.as_join().expect("join message should be valid");
        //         if self.is_our_name(join.user) {
        //             let chatters = self
        //                 .client
        //                 .get_chatters_for(join.channel.strip_prefix('#').unwrap_or(join.channel))
        //                 .unwrap();

        //             self.app
        //                 .tabs
        //                 .get_mut(join.channel)
        //                 .update_chatters(chatters);
        //             self.app.tabs.set_active_by_name(join.channel)
        //         }
        //     }

        //     Part => {
        //         let part = msg.as_part().expect("part message should be valid");
        //         if self.is_our_name(part.user) {
        //             self.app.tabs.remove_tab(part.channel);
        //         }
        //     }

        //     Privmsg => {
        //         let pm = msg.as_privmsg().expect("privmsg message should be valid");
        //         let color = pm.color();
        //         let spans = vec![];

        //         // let spans = pm
        //         //     .emote_span()
        //         //     .into_iter()
        //         //     .map(|kind| match kind {
        //         //         twitch::TextKind::Text(inner) => {
        //         //             twitch::TextKind::Text(Cow::Owned(inner.to_string()))
        //         //         }
        //         //         twitch::TextKind::Emote(id) => twitch::TextKind::Emote(id),
        //         //     })
        //         //     .collect();

        //         let line = TwitchLine::new(
        //             pm.sender, pm.target, //
        //             pm.data, spans,
        //         )
        //         .with_color(color);
        //         self.app
        //             .tabs
        //             .get_mut(&line.source)
        //             .append(tabs::Line::Twitch { line });
        //     }

        // _ => {}
        // }

        true
    }

    const fn is_connected(&self) -> bool {
        self.app.twitch.is_some()
    }

    fn toggle_tab_bar(&mut self) {
        // self.app.showing_tab_bar = !self.app.showing_tab_bar;
    }

    fn toggle_user_list(&mut self) {
        // self.app.tabs.active_mut().toggle_user_list()
    }

    fn toggle_line_mode(&mut self) {
        // self.app.tabs.active_mut().next_line_mode()
    }

    fn toggle_timestamps(&mut self) {
        // self.app.tabs.active_mut().toggle_timestamps()
    }

    fn next_tab(&mut self) {
        // self.app.tabs.next_tab()
    }

    fn previous_tab(&mut self) {
        // self.app.tabs.previous_tab()
    }

    fn try_set_active_tab(&mut self, index: usize) {
        // self.app.tabs.set_active(index);
    }

    fn switch_to_settings(&mut self) {
        self.switch_to_view(MainViewView::Settings)
    }

    fn switch_to_main(&mut self) {
        self.switch_to_view(self.app.state.previous_view)
    }

    fn switch_to_view(&mut self, view: MainViewView) {
        if self.app.state.current_view == view {
            return;
        }
        self.app.state.previous_view = std::mem::replace(&mut self.app.state.current_view, view);
    }

    fn try_handle_key_press(&mut self) {
        if self.context.input().events.is_empty() {
            return;
        }

        if self.context.input().key_pressed(egui::Key::F12) {
            self.context.set_debug_on_hover(
                !self.context.debug_on_hover(), //
            )
        }

        if self.app.state.keybind_state.is_capturing() {
            return;
        }

        let ctx = self.context.clone();
        for (key, modifiers) in ctx.input().events.iter().filter_map(|c| match c {
            &egui::Event::Key {
                key,
                pressed,
                modifiers,
            } if !pressed => Some((key, modifiers)),
            _ => None,
        }) {
            if let Some(action) = self.app.state.key_mapping.find(key, modifiers) {
                use crate::KeyAction::*;

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

        egui::CentralPanel::default().show(ctx, |ui| MainView::new(&mut self.app).display(ui));
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
